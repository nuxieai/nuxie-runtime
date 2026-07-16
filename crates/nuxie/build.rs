use std::{env, fmt::Write as _, fs, path::PathBuf};

use nuxie_schema::{
    Definition, FieldKind, Property, definition_by_name, property_by_key_in_hierarchy,
};

#[derive(Clone, Copy)]
struct FieldSpec {
    rust_name: &'static str,
    schema_name: &'static str,
    declared_owner: &'static str,
    kind: FieldKind,
    inherited: bool,
}

#[derive(Clone, Copy)]
struct ObjectSpec {
    rust_name: &'static str,
    schema_name: &'static str,
    fields: &'static [FieldSpec],
    is_node: bool,
}

#[derive(Clone, Copy)]
struct PropSpec {
    rust_name: &'static str,
    schema_name: &'static str,
    declared_owner: &'static str,
    kind: FieldKind,
}

const NAME: FieldSpec = FieldSpec {
    rust_name: "name",
    schema_name: "name",
    declared_owner: "Component",
    kind: FieldKind::String,
    inherited: true,
};

const ASSET_NAME: FieldSpec = FieldSpec {
    rust_name: "name",
    schema_name: "name",
    declared_owner: "Asset",
    kind: FieldKind::String,
    inherited: true,
};

const OBJECTS: &[ObjectSpec] = &[
    ObjectSpec {
        rust_name: "Artboard",
        schema_name: "Artboard",
        fields: &[
            NAME,
            FieldSpec {
                rust_name: "width",
                schema_name: "width",
                declared_owner: "LayoutComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "height",
                schema_name: "height",
                declared_owner: "LayoutComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
        ],
        is_node: false,
    },
    ObjectSpec {
        rust_name: "Shape",
        schema_name: "Shape",
        fields: &[
            NAME,
            FieldSpec {
                rust_name: "x",
                schema_name: "x",
                declared_owner: "Node",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "y",
                schema_name: "y",
                declared_owner: "Node",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "opacity",
                schema_name: "opacity",
                declared_owner: "WorldTransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "rotation",
                schema_name: "rotation",
                declared_owner: "TransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "scale_x",
                schema_name: "scaleX",
                declared_owner: "TransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "scale_y",
                schema_name: "scaleY",
                declared_owner: "TransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
        ],
        is_node: true,
    },
    ObjectSpec {
        rust_name: "NestedArtboard",
        schema_name: "NestedArtboard",
        fields: &[
            NAME,
            FieldSpec {
                rust_name: "x",
                schema_name: "x",
                declared_owner: "Node",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "y",
                schema_name: "y",
                declared_owner: "Node",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "opacity",
                schema_name: "opacity",
                declared_owner: "WorldTransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "rotation",
                schema_name: "rotation",
                declared_owner: "TransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "scale_x",
                schema_name: "scaleX",
                declared_owner: "TransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "scale_y",
                schema_name: "scaleY",
                declared_owner: "TransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "artboard",
                schema_name: "artboardId",
                declared_owner: "NestedArtboard",
                kind: FieldKind::Uint,
                inherited: false,
            },
        ],
        is_node: true,
    },
    ObjectSpec {
        rust_name: "Image",
        schema_name: "Image",
        fields: &[
            NAME,
            FieldSpec {
                rust_name: "x",
                schema_name: "x",
                declared_owner: "Node",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "y",
                schema_name: "y",
                declared_owner: "Node",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "opacity",
                schema_name: "opacity",
                declared_owner: "WorldTransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "rotation",
                schema_name: "rotation",
                declared_owner: "TransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "scale_x",
                schema_name: "scaleX",
                declared_owner: "TransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "scale_y",
                schema_name: "scaleY",
                declared_owner: "TransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "image",
                schema_name: "assetId",
                declared_owner: "Image",
                kind: FieldKind::Uint,
                inherited: false,
            },
            FieldSpec {
                rust_name: "origin_x",
                schema_name: "originX",
                declared_owner: "Image",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "origin_y",
                schema_name: "originY",
                declared_owner: "Image",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "fit",
                schema_name: "fit",
                declared_owner: "Image",
                kind: FieldKind::Uint,
                inherited: false,
            },
            FieldSpec {
                rust_name: "alignment_x",
                schema_name: "alignmentX",
                declared_owner: "Image",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "alignment_y",
                schema_name: "alignmentY",
                declared_owner: "Image",
                kind: FieldKind::Double,
                inherited: false,
            },
        ],
        is_node: true,
    },
    ObjectSpec {
        rust_name: "Rectangle",
        schema_name: "Rectangle",
        fields: &[
            NAME,
            FieldSpec {
                rust_name: "width",
                schema_name: "width",
                declared_owner: "ParametricPath",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "height",
                schema_name: "height",
                declared_owner: "ParametricPath",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "corner_radius_tl",
                schema_name: "cornerRadiusTL",
                declared_owner: "Rectangle",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "corner_radius_tr",
                schema_name: "cornerRadiusTR",
                declared_owner: "Rectangle",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "corner_radius_br",
                schema_name: "cornerRadiusBR",
                declared_owner: "Rectangle",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "corner_radius_bl",
                schema_name: "cornerRadiusBL",
                declared_owner: "Rectangle",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "link_corner_radius",
                schema_name: "linkCornerRadius",
                declared_owner: "Rectangle",
                kind: FieldKind::Bool,
                inherited: false,
            },
        ],
        is_node: true,
    },
    ObjectSpec {
        rust_name: "Fill",
        schema_name: "Fill",
        fields: &[NAME],
        is_node: true,
    },
    ObjectSpec {
        rust_name: "SolidColor",
        schema_name: "SolidColor",
        fields: &[
            NAME,
            FieldSpec {
                rust_name: "color",
                schema_name: "colorValue",
                declared_owner: "SolidColor",
                kind: FieldKind::Color,
                inherited: false,
            },
        ],
        is_node: true,
    },
    ObjectSpec {
        rust_name: "Stroke",
        schema_name: "Stroke",
        fields: &[
            NAME,
            FieldSpec {
                rust_name: "thickness",
                schema_name: "thickness",
                declared_owner: "Stroke",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "cap",
                schema_name: "cap",
                declared_owner: "Stroke",
                kind: FieldKind::Uint,
                inherited: false,
            },
            FieldSpec {
                rust_name: "join",
                schema_name: "join",
                declared_owner: "Stroke",
                kind: FieldKind::Uint,
                inherited: false,
            },
            FieldSpec {
                rust_name: "transform_affects_stroke",
                schema_name: "transformAffectsStroke",
                declared_owner: "Stroke",
                kind: FieldKind::Bool,
                inherited: false,
            },
        ],
        is_node: true,
    },
    ObjectSpec {
        rust_name: "DashPath",
        schema_name: "DashPath",
        fields: &[
            NAME,
            FieldSpec {
                rust_name: "offset",
                schema_name: "offset",
                declared_owner: "DashPath",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "offset_is_percentage",
                schema_name: "offsetIsPercentage",
                declared_owner: "DashPath",
                kind: FieldKind::Bool,
                inherited: false,
            },
        ],
        is_node: true,
    },
    ObjectSpec {
        rust_name: "Dash",
        schema_name: "Dash",
        fields: &[
            NAME,
            FieldSpec {
                rust_name: "length",
                schema_name: "length",
                declared_owner: "Dash",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "length_is_percentage",
                schema_name: "lengthIsPercentage",
                declared_owner: "Dash",
                kind: FieldKind::Bool,
                inherited: false,
            },
        ],
        is_node: true,
    },
    ObjectSpec {
        rust_name: "Text",
        schema_name: "Text",
        fields: &[
            NAME,
            FieldSpec {
                rust_name: "x",
                schema_name: "x",
                declared_owner: "Node",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "y",
                schema_name: "y",
                declared_owner: "Node",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "opacity",
                schema_name: "opacity",
                declared_owner: "WorldTransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "rotation",
                schema_name: "rotation",
                declared_owner: "TransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "scale_x",
                schema_name: "scaleX",
                declared_owner: "TransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "scale_y",
                schema_name: "scaleY",
                declared_owner: "TransformComponent",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "sizing",
                schema_name: "sizingValue",
                declared_owner: "Text",
                kind: FieldKind::Uint,
                inherited: false,
            },
            FieldSpec {
                rust_name: "width",
                schema_name: "width",
                declared_owner: "Text",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "height",
                schema_name: "height",
                declared_owner: "Text",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "align",
                schema_name: "alignValue",
                declared_owner: "Text",
                kind: FieldKind::Uint,
                inherited: false,
            },
            FieldSpec {
                rust_name: "wrap",
                schema_name: "wrapValue",
                declared_owner: "Text",
                kind: FieldKind::Uint,
                inherited: false,
            },
            FieldSpec {
                rust_name: "overflow",
                schema_name: "overflowValue",
                declared_owner: "Text",
                kind: FieldKind::Uint,
                inherited: false,
            },
        ],
        is_node: true,
    },
    ObjectSpec {
        rust_name: "TextValueRun",
        schema_name: "TextValueRun",
        fields: &[
            NAME,
            FieldSpec {
                rust_name: "text",
                schema_name: "text",
                declared_owner: "TextValueRun",
                kind: FieldKind::String,
                inherited: false,
            },
            FieldSpec {
                rust_name: "style",
                schema_name: "styleId",
                declared_owner: "TextValueRun",
                kind: FieldKind::Uint,
                inherited: false,
            },
        ],
        is_node: true,
    },
    ObjectSpec {
        rust_name: "TextStylePaint",
        schema_name: "TextStylePaint",
        fields: &[
            NAME,
            FieldSpec {
                rust_name: "font_size",
                schema_name: "fontSize",
                declared_owner: "TextStyle",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "line_height",
                schema_name: "lineHeight",
                declared_owner: "TextStyle",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "letter_spacing",
                schema_name: "letterSpacing",
                declared_owner: "TextStyle",
                kind: FieldKind::Double,
                inherited: true,
            },
            FieldSpec {
                rust_name: "font",
                schema_name: "fontAssetId",
                declared_owner: "TextStyle",
                kind: FieldKind::Uint,
                inherited: true,
            },
        ],
        is_node: true,
    },
    ObjectSpec {
        rust_name: "FontAsset",
        schema_name: "FontAsset",
        fields: &[ASSET_NAME],
        is_node: false,
    },
    ObjectSpec {
        rust_name: "ImageAsset",
        schema_name: "ImageAsset",
        fields: &[ASSET_NAME],
        is_node: false,
    },
];

// The first cursor surface is deliberately narrower than the structural
// NodeSpec vocabulary. Border topology (including the optional all-radii
// block and dash children) is structural in E2 and is replaced in one scene
// edit; it must not gain per-field Prop tokens until a presence/update policy
// for that aggregate is pinned. Transforms, opacity, rectangle width, and
// solid color are the current hot-write surface.
const PROPS: &[PropSpec] = &[
    PropSpec {
        rust_name: "PATH_WIDTH",
        schema_name: "width",
        declared_owner: "ParametricPath",
        kind: FieldKind::Double,
    },
    PropSpec {
        rust_name: "COLOR_VALUE",
        schema_name: "colorValue",
        declared_owner: "SolidColor",
        kind: FieldKind::Color,
    },
    PropSpec {
        rust_name: "WORLD_OPACITY",
        schema_name: "opacity",
        declared_owner: "WorldTransformComponent",
        kind: FieldKind::Double,
    },
    PropSpec {
        rust_name: "TRANSLATE_X",
        schema_name: "x",
        declared_owner: "Node",
        kind: FieldKind::Double,
    },
    PropSpec {
        rust_name: "TRANSLATE_Y",
        schema_name: "y",
        declared_owner: "Node",
        kind: FieldKind::Double,
    },
    PropSpec {
        rust_name: "ROTATION",
        schema_name: "rotation",
        declared_owner: "TransformComponent",
        kind: FieldKind::Double,
    },
    PropSpec {
        rust_name: "SCALE_X",
        schema_name: "scaleX",
        declared_owner: "TransformComponent",
        kind: FieldKind::Double,
    },
    PropSpec {
        rust_name: "SCALE_Y",
        schema_name: "scaleY",
        declared_owner: "TransformComponent",
        kind: FieldKind::Double,
    },
];

fn main() {
    println!("cargo:rerun-if-changed=../nuxie-schema/src/generated/schema.rs");
    println!("cargo:rerun-if-changed=../nuxie-schema/src/lib.rs");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("Cargo provides OUT_DIR"));
    fs::write(out_dir.join("scene_schema.rs"), render_scene_schema())
        .expect("write generated Scene authoring vocabulary");
}

fn render_scene_schema() -> String {
    let backboard = concrete_definition("Backboard");
    let file_asset_contents = concrete_definition("FileAssetContents");
    let mesh = concrete_definition("Mesh");
    let mesh_vertex = concrete_definition("MeshVertex");
    let mut output =
        String::from("// @generated by crates/nuxie/build.rs from nuxie-schema; do not edit.\n\n");

    writeln!(
        output,
        "pub(super) const TYPE_BACKBOARD: u16 = {};",
        backboard.type_key.int
    )
    .expect("write generated source");
    writeln!(
        output,
        "pub(super) const TYPE_FILE_ASSET_CONTENTS: u16 = {};",
        file_asset_contents.type_key.int
    )
    .expect("write generated source");
    writeln!(
        output,
        "pub(super) const TYPE_MESH: u16 = {};",
        mesh.type_key.int
    )
    .expect("write generated source");
    writeln!(
        output,
        "pub(super) const TYPE_MESH_VERTEX: u16 = {};",
        mesh_vertex.type_key.int
    )
    .expect("write generated source");

    for object in OBJECTS {
        let definition = concrete_definition(object.schema_name);
        writeln!(
            output,
            "pub(super) const TYPE_{}: u16 = {};",
            screaming_snake(object.rust_name),
            definition.type_key.int
        )
        .expect("write generated source");
        for field in object.fields {
            let _ = resolve_property(object.schema_name, *field);
        }
    }

    let component_name = resolve_property("Shape", NAME);
    let parent_id = resolve_property(
        "Shape",
        FieldSpec {
            rust_name: "parent_id",
            schema_name: "parentId",
            declared_owner: "Component",
            kind: FieldKind::Uint,
            inherited: true,
        },
    );
    let artboard_width = resolve_named_property(
        "Artboard",
        "width",
        "LayoutComponent",
        FieldKind::Double,
        true,
    );
    let artboard_height = resolve_named_property(
        "Artboard",
        "height",
        "LayoutComponent",
        FieldKind::Double,
        true,
    );
    let translate_x = resolve_named_property("Shape", "x", "Node", FieldKind::Double, true);
    let translate_y = resolve_named_property("Shape", "y", "Node", FieldKind::Double, true);
    let shape_opacity = resolve_named_property(
        "Shape",
        "opacity",
        "WorldTransformComponent",
        FieldKind::Double,
        true,
    );
    let rotation = resolve_named_property(
        "Shape",
        "rotation",
        "TransformComponent",
        FieldKind::Double,
        true,
    );
    let scale_x = resolve_named_property(
        "Shape",
        "scaleX",
        "TransformComponent",
        FieldKind::Double,
        true,
    );
    let scale_y = resolve_named_property(
        "Shape",
        "scaleY",
        "TransformComponent",
        FieldKind::Double,
        true,
    );
    let nested_artboard_id = resolve_named_property(
        "NestedArtboard",
        "artboardId",
        "NestedArtboard",
        FieldKind::Uint,
        false,
    );
    let image_asset_id =
        resolve_named_property("Image", "assetId", "Image", FieldKind::Uint, false);
    let image_origin_x =
        resolve_named_property("Image", "originX", "Image", FieldKind::Double, false);
    let image_origin_y =
        resolve_named_property("Image", "originY", "Image", FieldKind::Double, false);
    let image_fit = resolve_named_property("Image", "fit", "Image", FieldKind::Uint, false);
    let image_alignment_x =
        resolve_named_property("Image", "alignmentX", "Image", FieldKind::Double, false);
    let image_alignment_y =
        resolve_named_property("Image", "alignmentY", "Image", FieldKind::Double, false);
    let path_width = resolve_named_property(
        "Rectangle",
        "width",
        "ParametricPath",
        FieldKind::Double,
        true,
    );
    let path_height = resolve_named_property(
        "Rectangle",
        "height",
        "ParametricPath",
        FieldKind::Double,
        true,
    );
    let fill_rule = resolve_named_property("Fill", "fillRule", "Fill", FieldKind::Uint, false);
    let color_value = resolve_named_property(
        "SolidColor",
        "colorValue",
        "SolidColor",
        FieldKind::Color,
        false,
    );
    let rectangle_corner_radius_tl = resolve_named_property(
        "Rectangle",
        "cornerRadiusTL",
        "Rectangle",
        FieldKind::Double,
        false,
    );
    let rectangle_corner_radius_tr = resolve_named_property(
        "Rectangle",
        "cornerRadiusTR",
        "Rectangle",
        FieldKind::Double,
        false,
    );
    let rectangle_corner_radius_br = resolve_named_property(
        "Rectangle",
        "cornerRadiusBR",
        "Rectangle",
        FieldKind::Double,
        false,
    );
    let rectangle_corner_radius_bl = resolve_named_property(
        "Rectangle",
        "cornerRadiusBL",
        "Rectangle",
        FieldKind::Double,
        false,
    );
    let rectangle_link_corner_radius = resolve_named_property(
        "Rectangle",
        "linkCornerRadius",
        "Rectangle",
        FieldKind::Bool,
        false,
    );
    let stroke_thickness =
        resolve_named_property("Stroke", "thickness", "Stroke", FieldKind::Double, false);
    let stroke_cap = resolve_named_property("Stroke", "cap", "Stroke", FieldKind::Uint, false);
    let stroke_join = resolve_named_property("Stroke", "join", "Stroke", FieldKind::Uint, false);
    let stroke_transform_affects_stroke = resolve_named_property(
        "Stroke",
        "transformAffectsStroke",
        "Stroke",
        FieldKind::Bool,
        false,
    );
    let dash_offset =
        resolve_named_property("DashPath", "offset", "DashPath", FieldKind::Double, false);
    let dash_offset_is_percentage = resolve_named_property(
        "DashPath",
        "offsetIsPercentage",
        "DashPath",
        FieldKind::Bool,
        false,
    );
    let dash_length = resolve_named_property("Dash", "length", "Dash", FieldKind::Double, false);
    let dash_length_is_percentage =
        resolve_named_property("Dash", "lengthIsPercentage", "Dash", FieldKind::Bool, false);
    let asset_name = resolve_named_property("FontAsset", "name", "Asset", FieldKind::String, true);
    let file_asset_id =
        resolve_named_property("FontAsset", "assetId", "FileAsset", FieldKind::Uint, true);
    let file_asset_contents_bytes = resolve_encoded_property(
        "FileAssetContents",
        "bytes",
        "FileAssetContents",
        FieldKind::Bytes,
    );
    let text_sizing = resolve_named_property("Text", "sizingValue", "Text", FieldKind::Uint, false);
    let text_align = resolve_named_property("Text", "alignValue", "Text", FieldKind::Uint, false);
    let text_width = resolve_named_property("Text", "width", "Text", FieldKind::Double, false);
    let text_height = resolve_named_property("Text", "height", "Text", FieldKind::Double, false);
    let text_wrap = resolve_named_property("Text", "wrapValue", "Text", FieldKind::Uint, false);
    let text_overflow =
        resolve_named_property("Text", "overflowValue", "Text", FieldKind::Uint, false);
    let text_value_run_text = resolve_named_property(
        "TextValueRun",
        "text",
        "TextValueRun",
        FieldKind::String,
        false,
    );
    let text_value_run_style_id = resolve_named_property(
        "TextValueRun",
        "styleId",
        "TextValueRun",
        FieldKind::Uint,
        false,
    );
    let text_style_font_size = resolve_named_property(
        "TextStylePaint",
        "fontSize",
        "TextStyle",
        FieldKind::Double,
        true,
    );
    let text_style_line_height = resolve_named_property(
        "TextStylePaint",
        "lineHeight",
        "TextStyle",
        FieldKind::Double,
        true,
    );
    let text_style_letter_spacing = resolve_named_property(
        "TextStylePaint",
        "letterSpacing",
        "TextStyle",
        FieldKind::Double,
        true,
    );
    let text_style_font_asset_id = resolve_named_property(
        "TextStylePaint",
        "fontAssetId",
        "TextStyle",
        FieldKind::Uint,
        true,
    );
    let mesh_triangle_index_bytes =
        resolve_encoded_property("Mesh", "triangleIndexBytes", "Mesh", FieldKind::Bytes);
    let vertex_x = resolve_named_property("MeshVertex", "x", "Vertex", FieldKind::Double, true);
    let vertex_y = resolve_named_property("MeshVertex", "y", "Vertex", FieldKind::Double, true);
    let mesh_vertex_u =
        resolve_named_property("MeshVertex", "u", "MeshVertex", FieldKind::Double, false);
    let mesh_vertex_v =
        resolve_named_property("MeshVertex", "v", "MeshVertex", FieldKind::Double, false);
    for (name, property) in [
        ("COMPONENT_NAME", component_name),
        ("PARENT_ID", parent_id),
        ("LAYOUT_WIDTH", artboard_width),
        ("LAYOUT_HEIGHT", artboard_height),
        ("TRANSLATE_X", translate_x),
        ("TRANSLATE_Y", translate_y),
        ("WORLD_OPACITY", shape_opacity),
        ("ROTATION", rotation),
        ("SCALE_X", scale_x),
        ("SCALE_Y", scale_y),
        ("NESTED_ARTBOARD_ID", nested_artboard_id),
        ("IMAGE_ASSET_ID", image_asset_id),
        ("IMAGE_ORIGIN_X", image_origin_x),
        ("IMAGE_ORIGIN_Y", image_origin_y),
        ("IMAGE_FIT", image_fit),
        ("IMAGE_ALIGNMENT_X", image_alignment_x),
        ("IMAGE_ALIGNMENT_Y", image_alignment_y),
        ("PATH_WIDTH", path_width),
        ("PATH_HEIGHT", path_height),
        ("FILL_RULE", fill_rule),
        ("COLOR_VALUE", color_value),
        ("RECTANGLE_CORNER_RADIUS_TL", rectangle_corner_radius_tl),
        ("RECTANGLE_CORNER_RADIUS_TR", rectangle_corner_radius_tr),
        ("RECTANGLE_CORNER_RADIUS_BR", rectangle_corner_radius_br),
        ("RECTANGLE_CORNER_RADIUS_BL", rectangle_corner_radius_bl),
        ("RECTANGLE_LINK_CORNER_RADIUS", rectangle_link_corner_radius),
        ("STROKE_THICKNESS", stroke_thickness),
        ("STROKE_CAP", stroke_cap),
        ("STROKE_JOIN", stroke_join),
        (
            "STROKE_TRANSFORM_AFFECTS_STROKE",
            stroke_transform_affects_stroke,
        ),
        ("DASH_OFFSET", dash_offset),
        ("DASH_OFFSET_IS_PERCENTAGE", dash_offset_is_percentage),
        ("DASH_LENGTH", dash_length),
        ("DASH_LENGTH_IS_PERCENTAGE", dash_length_is_percentage),
        ("ASSET_NAME", asset_name),
        ("FILE_ASSET_ID", file_asset_id),
        ("FILE_ASSET_CONTENTS_BYTES", file_asset_contents_bytes),
        ("TEXT_SIZING", text_sizing),
        ("TEXT_ALIGN", text_align),
        ("TEXT_WIDTH", text_width),
        ("TEXT_HEIGHT", text_height),
        ("TEXT_WRAP", text_wrap),
        ("TEXT_OVERFLOW", text_overflow),
        ("TEXT_VALUE_RUN_TEXT", text_value_run_text),
        ("TEXT_VALUE_RUN_STYLE_ID", text_value_run_style_id),
        ("TEXT_STYLE_FONT_SIZE", text_style_font_size),
        ("TEXT_STYLE_LINE_HEIGHT", text_style_line_height),
        ("TEXT_STYLE_LETTER_SPACING", text_style_letter_spacing),
        ("TEXT_STYLE_FONT_ASSET_ID", text_style_font_asset_id),
        ("MESH_TRIANGLE_INDEX_BYTES", mesh_triangle_index_bytes),
        ("VERTEX_X", vertex_x),
        ("VERTEX_Y", vertex_y),
        ("MESH_VERTEX_U", mesh_vertex_u),
        ("MESH_VERTEX_V", mesh_vertex_v),
    ] {
        writeln!(
            output,
            "pub(super) const PROPERTY_{name}: u16 = {};",
            property.key.int
        )
        .expect("write generated source");
    }

    output.push_str(
        "\n#[derive(Debug, Clone, Copy, PartialEq)]\n\
         pub struct ImageCropRect {\n\
             pub x: f32,\n\
             pub y: f32,\n\
             pub width: f32,\n\
             pub height: f32,\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq)]\n\
         pub struct RectangleCornerRadii {\n\
             pub top_left: f32,\n\
             pub top_right: f32,\n\
             pub bottom_right: f32,\n\
             pub bottom_left: f32,\n\
             pub linked: bool,\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n\
         pub enum SceneTextSizing {\n\
             Fixed,\n\
         }\n\n\
         impl SceneTextSizing {\n\
             const fn wire_value(self) -> u32 {\n\
                 match self {\n\
                     Self::Fixed => 2,\n\
                 }\n\
             }\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n\
         pub enum SceneTextAlign {\n\
             Left,\n\
             Right,\n\
             Center,\n\
         }\n\n\
         impl SceneTextAlign {\n\
             const fn wire_value(self) -> u32 {\n\
                 match self {\n\
                     Self::Left => 0,\n\
                     Self::Right => 1,\n\
                     Self::Center => 2,\n\
                 }\n\
             }\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n\
         pub enum SceneTextWrap {\n\
             Wrap,\n\
             NoWrap,\n\
         }\n\n\
         impl SceneTextWrap {\n\
             const fn wire_value(self) -> u32 {\n\
                 match self {\n\
                     Self::Wrap => 0,\n\
                     Self::NoWrap => 1,\n\
                 }\n\
             }\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n\
         pub enum SceneTextOverflow {\n\
             Visible,\n\
             Hidden,\n\
             Clipped,\n\
             Ellipsis,\n\
             Fit,\n\
         }\n\n\
         impl SceneTextOverflow {\n\
             const fn wire_value(self) -> u32 {\n\
                 match self {\n\
                     Self::Visible => 0,\n\
                     Self::Hidden => 1,\n\
                     Self::Clipped => 2,\n\
                     Self::Ellipsis => 3,\n\
                     Self::Fit => 4,\n\
                 }\n\
             }\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n\
         pub enum SceneStrokeCap {\n\
             Butt,\n\
             Round,\n\
             Square,\n\
         }\n\n\
         impl SceneStrokeCap {\n\
             const fn wire_value(self) -> u32 {\n\
                 match self {\n\
                     Self::Butt => 0,\n\
                     Self::Round => 1,\n\
                     Self::Square => 2,\n\
                 }\n\
             }\n\
         }\n\n\
         #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n\
         pub enum SceneStrokeJoin {\n\
             Miter,\n\
             Round,\n\
             Bevel,\n\
         }\n\n\
         impl SceneStrokeJoin {\n\
             const fn wire_value(self) -> u32 {\n\
                 match self {\n\
                     Self::Miter => 0,\n\
                     Self::Round => 1,\n\
                     Self::Bevel => 2,\n\
                 }\n\
             }\n\
         }\n\n",
    );
    for object in OBJECTS {
        writeln!(output, "#[derive(Debug, Clone, PartialEq)]").expect("write generated source");
        writeln!(output, "pub struct {}Spec {{", object.rust_name).expect("write generated source");
        for field in object.fields {
            let property = resolve_property(object.schema_name, *field);
            if is_rectangle_corner_radius_field(object, field) {
                continue;
            }
            writeln!(
                output,
                "    pub {}: {},",
                field.rust_name,
                public_field_rust_type(object, field, property.runtime_type)
            )
            .expect("write generated source");
        }
        if object.rust_name == "Rectangle" {
            output.push_str("    pub corner_radii: Option<RectangleCornerRadii>,\n");
        }
        if object.rust_name == "Image" {
            output.push_str("    pub crop: Option<ImageCropRect>,\n");
        }
        if object.rust_name == "FontAsset" || object.rust_name == "ImageAsset" {
            output.push_str("    pub bytes: Vec<u8>,\n");
        }
        output.push_str("}\n\n");
    }
    output.push_str(
        "impl RectangleSpec {\n\
             pub fn new(name: impl Into<String>, width: f32, height: f32) -> Self {\n\
                 Self {\n\
                     name: name.into(),\n\
                     width,\n\
                     height,\n\
                     corner_radii: None,\n\
                 }\n\
             }\n\
         }\n\n",
    );

    output.push_str(
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n\
         pub enum NodeKind {\n",
    );
    for object in OBJECTS.iter().filter(|object| object.is_node) {
        writeln!(output, "    {},", object.rust_name).expect("write generated source");
    }
    output.push_str("}\n\n");
    output.push_str(
        "impl NodeKind {\n\
             pub const fn schema_name(self) -> &'static str {\n\
                 match self {\n",
    );
    for object in OBJECTS.iter().filter(|object| object.is_node) {
        writeln!(
            output,
            "            Self::{} => {:?},",
            object.rust_name, object.schema_name,
        )
        .expect("write generated source");
    }
    output.push_str("        }\n    }\n}\n\n");

    output.push_str("#[derive(Debug, Clone, PartialEq)]\npub enum NodeSpec {\n");
    for object in OBJECTS.iter().filter(|object| object.is_node) {
        writeln!(
            output,
            "    {}({}Spec),",
            object.rust_name, object.rust_name
        )
        .expect("write generated source");
    }
    output.push_str(
        "}\n\nimpl NodeSpec {\n    pub const fn kind(&self) -> NodeKind {\n        match self {\n",
    );
    for object in OBJECTS.iter().filter(|object| object.is_node) {
        writeln!(
            output,
            "            Self::{}(_) => NodeKind::{},",
            object.rust_name, object.rust_name
        )
        .expect("write generated source");
    }
    output.push_str("        }\n    }\n}\n\n");

    for prop in PROPS {
        let resolved = resolve_declared_property(prop);
        let compatible = compatible_fields(prop, resolved);
        let function_name = prop.rust_name.to_ascii_lowercase();
        let supported_kinds = compatible
            .iter()
            .map(|(object, _)| format!("NodeKind::{}", object.rust_name))
            .collect::<Vec<_>>()
            .join(" | ");

        writeln!(
            output,
            "fn prop_{function_name}_is_available_on(kind: NodeKind) -> bool {{\n    matches!(kind, {supported_kinds})\n}}"
        )
        .expect("write generated source");
        writeln!(
            output,
            "fn prop_{function_name}_apply(\n    node: &mut NodeSpec,\n    value: {},\n) -> std::result::Result<(), EditReason> {{",
            rust_type(resolved.runtime_type)
        )
        .expect("write generated source");
        if resolved.runtime_type == FieldKind::Double {
            let first_field_name = if let Some((_, field)) = compatible.first() {
                field.rust_name
            } else {
                std::process::abort();
            };
            writeln!(
                output,
                "    if !value.is_finite() {{\n        return Err(EditReason::NonFiniteProperty {{ property: {:?} }});\n    }}",
                first_field_name
            )
            .expect("write generated source");
        }
        output.push_str("    match node {\n");
        for (object, field) in &compatible {
            writeln!(
                output,
                "        NodeSpec::{}(spec) => {{ spec.{} = value; Ok(()) }},",
                object.rust_name, field.rust_name
            )
            .expect("write generated source");
        }
        output.push_str("        _ => Err(EditReason::InternalInvariant),\n    }\n}\n\n");
    }

    output.push_str(
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub enum PropValueKind {\n\
             Double,\n\
             Color,\n\
         }\n\n\
         pub mod props {\n\
             use super::{Prop, PropValueKind, read_runtime_color, read_runtime_double, set_runtime_color, set_runtime_double};\n\
             use std::marker::PhantomData;\n\n",
    );
    for prop in PROPS {
        let resolved = resolve_declared_property(prop);
        let function_name = prop.rust_name.to_ascii_lowercase();
        writeln!(
            output,
            "    pub const {}: Prop<{}> = Prop {{\n        key: {},\n        schema_name: {:?},\n        value_kind: PropValueKind::{},\n        declared_owner: {:?},\n        is_available_on: super::prop_{}_is_available_on,\n        apply_to_definition: super::prop_{}_apply,\n        apply_to_runtime: {},\n        read_from_runtime: {},\n        marker: PhantomData,\n    }};",
            prop.rust_name,
            rust_type(resolved.runtime_type),
            resolved.key.int,
            prop.schema_name,
            value_kind_variant(resolved.runtime_type),
            prop.declared_owner,
            function_name,
            function_name,
            runtime_setter(resolved.runtime_type),
            runtime_reader(resolved.runtime_type),
        )
        .expect("write generated source");
    }
    output.push_str("}\n");
    output
}

fn concrete_definition(name: &str) -> &'static Definition {
    let definition = definition_by_name(name)
        .unwrap_or_else(|| panic!("authoring schema definition {name} must exist"));
    assert!(
        !definition.abstract_,
        "authoring schema definition {name} unexpectedly became abstract"
    );
    definition
}

fn resolve_property(target: &str, field: FieldSpec) -> &'static Property {
    resolve_named_property(
        target,
        field.schema_name,
        field.declared_owner,
        field.kind,
        field.inherited,
    )
}

fn resolve_named_property(
    target: &str,
    name: &str,
    declared_owner: &str,
    expected_kind: FieldKind,
    expected_inherited: bool,
) -> &'static Property {
    let target_definition = definition_by_name(target)
        .unwrap_or_else(|| panic!("authoring target schema definition {target} must exist"));
    let owner_definition = definition_by_name(declared_owner).unwrap_or_else(|| {
        panic!("authoring property owner schema definition {declared_owner} must exist")
    });
    let property = owner_definition
        .properties
        .iter()
        .find(|property| property.name == name)
        .unwrap_or_else(|| {
            panic!("authoring property {declared_owner}.{name} must remain directly declared")
        });
    assert_eq!(
        property.runtime_type, expected_kind,
        "authoring property {declared_owner}.{name} changed runtime value kind"
    );
    assert!(
        property.stores_data
            && property.deserializes
            && property.stores_field
            && !property.encoded
            && property.bitmask_passthrough.is_none()
            && property.cpp_generates_value_setter_body()
            && property.cpp_setter_uses_stored_field(),
        "authoring property {declared_owner}.{name} is no longer a directly writable stored field"
    );
    let (actual_owner, supported_property) =
        property_by_key_in_hierarchy(target_definition.type_key.int, property.key.int)
            .unwrap_or_else(|| {
                panic!("authoring target {target} no longer supports {declared_owner}.{name}")
            });
    assert_eq!(
        actual_owner, declared_owner,
        "authoring target {target} resolves {name} through a different schema owner"
    );
    assert_eq!(
        supported_property.runtime_type, expected_kind,
        "authoring target {target} resolves {name} with a different runtime value kind"
    );
    let inherited = target != declared_owner;
    assert_eq!(
        inherited, expected_inherited,
        "authoring target {target} changed inherited-property support for {declared_owner}.{name}"
    );
    property
}

fn resolve_encoded_property(
    target: &str,
    name: &str,
    declared_owner: &str,
    expected_kind: FieldKind,
) -> &'static Property {
    let target_definition = definition_by_name(target)
        .unwrap_or_else(|| panic!("authoring target schema definition {target} must exist"));
    let owner_definition = definition_by_name(declared_owner).unwrap_or_else(|| {
        panic!("authoring property owner schema definition {declared_owner} must exist")
    });
    let property = owner_definition
        .properties
        .iter()
        .find(|property| property.name == name)
        .unwrap_or_else(|| {
            panic!("authoring property {declared_owner}.{name} must remain directly declared")
        });
    assert_eq!(
        property.runtime_type, expected_kind,
        "authoring property {declared_owner}.{name} changed runtime value kind"
    );
    assert!(
        property.stores_data && property.deserializes && property.encoded,
        "authoring property {declared_owner}.{name} is no longer encoded stored data"
    );
    let (actual_owner, supported_property) =
        property_by_key_in_hierarchy(target_definition.type_key.int, property.key.int)
            .unwrap_or_else(|| {
                panic!("authoring target {target} no longer supports {declared_owner}.{name}")
            });
    assert_eq!(actual_owner, declared_owner);
    assert_eq!(supported_property.runtime_type, expected_kind);
    property
}

fn resolve_declared_property(prop: &PropSpec) -> &'static Property {
    let owner = definition_by_name(prop.declared_owner).unwrap_or_else(|| {
        panic!(
            "authoring property owner schema definition {} must exist",
            prop.declared_owner
        )
    });
    let property = owner
        .properties
        .iter()
        .find(|property| property.name == prop.schema_name)
        .unwrap_or_else(|| {
            panic!(
                "authoring property {}.{} must remain directly declared",
                prop.declared_owner, prop.schema_name
            )
        });
    assert_eq!(
        property.runtime_type, prop.kind,
        "authoring property {}.{} changed runtime value kind",
        prop.declared_owner, prop.schema_name
    );
    assert!(
        property.stores_data
            && property.deserializes
            && property.stores_field
            && !property.encoded
            && property.bitmask_passthrough.is_none()
            && property.cpp_generates_value_setter_body()
            && property.cpp_setter_uses_stored_field(),
        "authoring property {}.{} is no longer a directly writable stored field",
        prop.declared_owner,
        prop.schema_name
    );
    property
}

fn compatible_fields(
    prop: &PropSpec,
    property: &'static Property,
) -> Vec<(&'static ObjectSpec, &'static FieldSpec)> {
    let compatible = OBJECTS
        .iter()
        .filter(|object| object.is_node)
        .flat_map(|object| {
            object.fields.iter().filter_map(move |field| {
                (resolve_property(object.schema_name, *field).key.int == property.key.int)
                    .then_some((object, field))
            })
        })
        .collect::<Vec<_>>();
    assert!(
        !compatible.is_empty(),
        "authoring property {}.{} has no schema-backed NodeSpec field",
        prop.declared_owner,
        prop.schema_name
    );
    compatible
}

fn rust_type(kind: FieldKind) -> &'static str {
    match kind {
        FieldKind::String => "String",
        FieldKind::Double => "f32",
        FieldKind::Color => "u32",
        FieldKind::Uint => "u32",
        FieldKind::Bool => "bool",
        other => panic!("unsupported public Scene spec field kind {other:?}"),
    }
}

fn is_rectangle_corner_radius_field(object: &ObjectSpec, field: &FieldSpec) -> bool {
    object.rust_name == "Rectangle"
        && matches!(
            field.rust_name,
            "corner_radius_tl"
                | "corner_radius_tr"
                | "corner_radius_br"
                | "corner_radius_bl"
                | "link_corner_radius"
        )
}

fn public_field_rust_type(
    object: &ObjectSpec,
    field: &FieldSpec,
    runtime_type: FieldKind,
) -> &'static str {
    match (object.rust_name, field.rust_name) {
        ("Stroke", "cap") => "SceneStrokeCap",
        ("Stroke", "join") => "SceneStrokeJoin",
        ("Text", "sizing") => "SceneTextSizing",
        ("Text", "align") => "SceneTextAlign",
        ("Text", "wrap") => "SceneTextWrap",
        ("Text", "overflow") => "SceneTextOverflow",
        ("TextValueRun", "style") => "ObjectId",
        ("TextStylePaint", "font") => "FontAssetId",
        ("NestedArtboard", "artboard") => "ArtboardId",
        ("Image", "image") => "ImageAssetId",
        _ => rust_type(runtime_type),
    }
}

fn value_kind_variant(kind: FieldKind) -> &'static str {
    match kind {
        FieldKind::Double => "Double",
        FieldKind::Color => "Color",
        other => panic!("unsupported public Scene property token kind {other:?}"),
    }
}

fn runtime_setter(kind: FieldKind) -> &'static str {
    match kind {
        FieldKind::Double => "set_runtime_double",
        FieldKind::Color => "set_runtime_color",
        other => panic!("unsupported public Scene runtime property kind {other:?}"),
    }
}

fn runtime_reader(kind: FieldKind) -> &'static str {
    match kind {
        FieldKind::Double => "read_runtime_double",
        FieldKind::Color => "read_runtime_color",
        other => panic!("unsupported public Scene runtime property kind {other:?}"),
    }
}

fn screaming_snake(value: &str) -> String {
    let mut output = String::new();
    for (index, character) in value.char_indices() {
        if index != 0 && character.is_ascii_uppercase() {
            output.push('_');
        }
        output.push(character.to_ascii_uppercase());
    }
    output
}
