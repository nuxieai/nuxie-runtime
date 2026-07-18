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
        rust_name: "ScriptedDrawable",
        schema_name: "ScriptedDrawable",
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
                rust_name: "script",
                schema_name: "scriptAssetId",
                declared_owner: "ScriptedDrawable",
                kind: FieldKind::Uint,
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
        rust_name: "LinearGradient",
        schema_name: "LinearGradient",
        fields: &[
            NAME,
            FieldSpec {
                rust_name: "start_x",
                schema_name: "startX",
                declared_owner: "LinearGradient",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "start_y",
                schema_name: "startY",
                declared_owner: "LinearGradient",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "end_x",
                schema_name: "endX",
                declared_owner: "LinearGradient",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "end_y",
                schema_name: "endY",
                declared_owner: "LinearGradient",
                kind: FieldKind::Double,
                inherited: false,
            },
            FieldSpec {
                rust_name: "opacity",
                schema_name: "opacity",
                declared_owner: "LinearGradient",
                kind: FieldKind::Double,
                inherited: false,
            },
        ],
        is_node: true,
    },
    ObjectSpec {
        rust_name: "GradientStop",
        schema_name: "GradientStop",
        fields: &[
            NAME,
            FieldSpec {
                rust_name: "color",
                schema_name: "colorValue",
                declared_owner: "GradientStop",
                kind: FieldKind::Color,
                inherited: false,
            },
            FieldSpec {
                rust_name: "position",
                schema_name: "position",
                declared_owner: "GradientStop",
                kind: FieldKind::Double,
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
    ObjectSpec {
        rust_name: "ScriptAsset",
        schema_name: "ScriptAsset",
        fields: &[
            ASSET_NAME,
            FieldSpec {
                rust_name: "is_module",
                schema_name: "isModule",
                declared_owner: "ScriptAsset",
                kind: FieldKind::Bool,
                inherited: false,
            },
        ],
        is_node: false,
    },
    ObjectSpec {
        rust_name: "ShaderAsset",
        schema_name: "ShaderAsset",
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
    let linear_animation = concrete_definition("LinearAnimation");
    let keyed_object = concrete_definition("KeyedObject");
    let keyed_property = concrete_definition("KeyedProperty");
    let key_frame_double = concrete_definition("KeyFrameDouble");
    let event = concrete_definition("Event");
    let state_machine = concrete_definition("StateMachine");
    let state_machine_trigger = concrete_definition("StateMachineTrigger");
    let state_machine_layer = concrete_definition("StateMachineLayer");
    let any_state = concrete_definition("AnyState");
    let entry_state = concrete_definition("EntryState");
    let exit_state = concrete_definition("ExitState");
    let animation_state = concrete_definition("AnimationState");
    let state_transition = concrete_definition("StateTransition");
    let transition_trigger_condition = concrete_definition("TransitionTriggerCondition");
    let state_machine_fire_event = concrete_definition("StateMachineFireEvent");
    let view_model = concrete_definition("ViewModel");
    let view_model_property_number = concrete_definition("ViewModelPropertyNumber");
    let view_model_instance = concrete_definition("ViewModelInstance");
    let view_model_instance_number = concrete_definition("ViewModelInstanceNumber");
    let data_bind_context = concrete_definition("DataBindContext");
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
    for (name, definition) in [
        ("LINEAR_ANIMATION", linear_animation),
        ("KEYED_OBJECT", keyed_object),
        ("KEYED_PROPERTY", keyed_property),
        ("KEY_FRAME_DOUBLE", key_frame_double),
        ("EVENT", event),
        ("STATE_MACHINE", state_machine),
        ("STATE_MACHINE_TRIGGER", state_machine_trigger),
        ("STATE_MACHINE_LAYER", state_machine_layer),
        ("ANY_STATE", any_state),
        ("ENTRY_STATE", entry_state),
        ("EXIT_STATE", exit_state),
        ("ANIMATION_STATE", animation_state),
        ("STATE_TRANSITION", state_transition),
        ("TRANSITION_TRIGGER_CONDITION", transition_trigger_condition),
        ("STATE_MACHINE_FIRE_EVENT", state_machine_fire_event),
        ("VIEW_MODEL", view_model),
        ("VIEW_MODEL_PROPERTY_NUMBER", view_model_property_number),
        ("VIEW_MODEL_INSTANCE", view_model_instance),
        ("VIEW_MODEL_INSTANCE_NUMBER", view_model_instance_number),
        ("DATA_BIND_CONTEXT", data_bind_context),
    ] {
        writeln!(
            output,
            "pub(super) const TYPE_{name}: u16 = {};",
            definition.type_key.int
        )
        .expect("write generated source");
    }

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
    let scripted_drawable_script_asset_id = resolve_named_property(
        "ScriptedDrawable",
        "scriptAssetId",
        "ScriptedDrawable",
        FieldKind::Uint,
        false,
    );
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
    let linear_gradient_start_x = resolve_named_property(
        "LinearGradient",
        "startX",
        "LinearGradient",
        FieldKind::Double,
        false,
    );
    let linear_gradient_start_y = resolve_named_property(
        "LinearGradient",
        "startY",
        "LinearGradient",
        FieldKind::Double,
        false,
    );
    let linear_gradient_end_x = resolve_named_property(
        "LinearGradient",
        "endX",
        "LinearGradient",
        FieldKind::Double,
        false,
    );
    let linear_gradient_end_y = resolve_named_property(
        "LinearGradient",
        "endY",
        "LinearGradient",
        FieldKind::Double,
        false,
    );
    let linear_gradient_opacity = resolve_named_property(
        "LinearGradient",
        "opacity",
        "LinearGradient",
        FieldKind::Double,
        false,
    );
    let gradient_stop_color_value = resolve_named_property(
        "GradientStop",
        "colorValue",
        "GradientStop",
        FieldKind::Color,
        false,
    );
    let gradient_stop_position = resolve_named_property(
        "GradientStop",
        "position",
        "GradientStop",
        FieldKind::Double,
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
    let script_asset_is_module = resolve_named_property(
        "ScriptAsset",
        "isModule",
        "ScriptAsset",
        FieldKind::Bool,
        false,
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
    let animation_name = resolve_named_property(
        "LinearAnimation",
        "name",
        "Animation",
        FieldKind::String,
        true,
    );
    let animation_fps = resolve_named_property(
        "LinearAnimation",
        "fps",
        "LinearAnimation",
        FieldKind::Uint,
        false,
    );
    let animation_duration = resolve_named_property(
        "LinearAnimation",
        "duration",
        "LinearAnimation",
        FieldKind::Uint,
        false,
    );
    let animation_speed = resolve_named_property(
        "LinearAnimation",
        "speed",
        "LinearAnimation",
        FieldKind::Double,
        false,
    );
    let animation_loop = resolve_named_property(
        "LinearAnimation",
        "loopValue",
        "LinearAnimation",
        FieldKind::Uint,
        false,
    );
    let animation_work_start = resolve_named_property(
        "LinearAnimation",
        "workStart",
        "LinearAnimation",
        FieldKind::Uint,
        false,
    );
    let animation_work_end = resolve_named_property(
        "LinearAnimation",
        "workEnd",
        "LinearAnimation",
        FieldKind::Uint,
        false,
    );
    let animation_enable_work_area = resolve_named_property(
        "LinearAnimation",
        "enableWorkArea",
        "LinearAnimation",
        FieldKind::Bool,
        false,
    );
    let animation_quantize = resolve_named_property(
        "LinearAnimation",
        "quantize",
        "LinearAnimation",
        FieldKind::Bool,
        false,
    );
    let keyed_object_id = resolve_named_property(
        "KeyedObject",
        "objectId",
        "KeyedObject",
        FieldKind::Uint,
        false,
    );
    let keyed_property_key = resolve_named_property(
        "KeyedProperty",
        "propertyKey",
        "KeyedProperty",
        FieldKind::Uint,
        false,
    );
    let key_frame =
        resolve_named_property("KeyFrameDouble", "frame", "KeyFrame", FieldKind::Uint, true);
    let key_frame_interpolation_type = resolve_named_property(
        "KeyFrameDouble",
        "interpolationType",
        "InterpolatingKeyFrame",
        FieldKind::Uint,
        true,
    );
    let key_frame_double_value = resolve_named_property(
        "KeyFrameDouble",
        "value",
        "KeyFrameDouble",
        FieldKind::Double,
        false,
    );
    let state_machine_component_name = resolve_named_property(
        "StateMachineTrigger",
        "name",
        "StateMachineComponent",
        FieldKind::String,
        true,
    );
    let state_animation_id = resolve_named_property(
        "AnimationState",
        "animationId",
        "AnimationState",
        FieldKind::Uint,
        false,
    );
    let state_speed = resolve_named_property(
        "AnimationState",
        "speed",
        "AdvanceableState",
        FieldKind::Double,
        true,
    );
    let state_to_id = resolve_named_property(
        "StateTransition",
        "stateToId",
        "StateTransition",
        FieldKind::Uint,
        false,
    );
    let state_transition_flags = resolve_named_property(
        "StateTransition",
        "flags",
        "StateTransition",
        FieldKind::Uint,
        false,
    );
    let state_transition_duration = resolve_named_property(
        "StateTransition",
        "duration",
        "StateTransition",
        FieldKind::Uint,
        false,
    );
    let state_transition_exit_time = resolve_named_property(
        "StateTransition",
        "exitTime",
        "StateTransition",
        FieldKind::Uint,
        false,
    );
    let state_transition_random_weight = resolve_named_property(
        "StateTransition",
        "randomWeight",
        "StateTransition",
        FieldKind::Uint,
        false,
    );
    let state_machine_input_id = resolve_named_property(
        "TransitionTriggerCondition",
        "inputId",
        "TransitionInputCondition",
        FieldKind::Uint,
        true,
    );
    let state_machine_event_id = resolve_named_property(
        "StateMachineFireEvent",
        "eventId",
        "StateMachineFireEvent",
        FieldKind::Uint,
        false,
    );
    let state_machine_fire_occurs = resolve_named_property(
        "StateMachineFireEvent",
        "occursValue",
        "StateMachineFireAction",
        FieldKind::Uint,
        true,
    );
    let view_model_component_name = resolve_named_property(
        "ViewModel",
        "name",
        "ViewModelComponent",
        FieldKind::String,
        true,
    );
    let view_model_instance_view_model_id = resolve_named_property(
        "ViewModelInstance",
        "viewModelId",
        "ViewModelInstance",
        FieldKind::Uint,
        false,
    );
    let view_model_instance_value_property_id = resolve_named_property(
        "ViewModelInstanceNumber",
        "viewModelPropertyId",
        "ViewModelInstanceValue",
        FieldKind::Uint,
        true,
    );
    let view_model_instance_number_value = resolve_named_property(
        "ViewModelInstanceNumber",
        "propertyValue",
        "ViewModelInstanceNumber",
        FieldKind::Double,
        false,
    );
    let artboard_view_model_id = resolve_named_property(
        "Artboard",
        "viewModelId",
        "Artboard",
        FieldKind::Uint,
        false,
    );
    let data_bind_property_key = resolve_named_property(
        "DataBindContext",
        "propertyKey",
        "DataBind",
        FieldKind::Uint,
        true,
    );
    let data_bind_flags = resolve_named_property(
        "DataBindContext",
        "flags",
        "DataBind",
        FieldKind::Uint,
        true,
    );
    let data_bind_source_path = resolve_encoded_property(
        "DataBindContext",
        "sourcePathIds",
        "DataBindContext",
        FieldKind::Bytes,
    );
    for (property, expected_default) in [
        (animation_fps, "60"),
        (animation_speed, "1"),
        (animation_loop, "0"),
        (animation_work_start, "-1"),
        (animation_work_end, "-1"),
        (animation_enable_work_area, "false"),
        (animation_quantize, "false"),
    ] {
        assert_eq!(
            property.initial_value,
            Some(expected_default),
            "sparse authored animation lowering relies on this schema default"
        );
    }
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
        (
            "SCRIPTED_DRAWABLE_SCRIPT_ASSET_ID",
            scripted_drawable_script_asset_id,
        ),
        ("PATH_WIDTH", path_width),
        ("PATH_HEIGHT", path_height),
        ("FILL_RULE", fill_rule),
        ("COLOR_VALUE", color_value),
        ("LINEAR_GRADIENT_START_X", linear_gradient_start_x),
        ("LINEAR_GRADIENT_START_Y", linear_gradient_start_y),
        ("LINEAR_GRADIENT_END_X", linear_gradient_end_x),
        ("LINEAR_GRADIENT_END_Y", linear_gradient_end_y),
        ("LINEAR_GRADIENT_OPACITY", linear_gradient_opacity),
        ("GRADIENT_STOP_COLOR_VALUE", gradient_stop_color_value),
        ("GRADIENT_STOP_POSITION", gradient_stop_position),
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
        ("SCRIPT_ASSET_IS_MODULE", script_asset_is_module),
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
        ("ANIMATION_NAME", animation_name),
        ("ANIMATION_FPS", animation_fps),
        ("ANIMATION_DURATION", animation_duration),
        ("ANIMATION_SPEED", animation_speed),
        ("ANIMATION_LOOP", animation_loop),
        ("ANIMATION_WORK_START", animation_work_start),
        ("ANIMATION_WORK_END", animation_work_end),
        ("ANIMATION_ENABLE_WORK_AREA", animation_enable_work_area),
        ("ANIMATION_QUANTIZE", animation_quantize),
        ("KEYED_OBJECT_ID", keyed_object_id),
        ("KEYED_PROPERTY_KEY", keyed_property_key),
        ("KEY_FRAME", key_frame),
        ("KEY_FRAME_INTERPOLATION_TYPE", key_frame_interpolation_type),
        ("KEY_FRAME_DOUBLE_VALUE", key_frame_double_value),
        ("STATE_MACHINE_COMPONENT_NAME", state_machine_component_name),
        ("STATE_ANIMATION_ID", state_animation_id),
        ("STATE_SPEED", state_speed),
        ("STATE_TO_ID", state_to_id),
        ("STATE_TRANSITION_FLAGS", state_transition_flags),
        ("STATE_TRANSITION_DURATION", state_transition_duration),
        ("STATE_TRANSITION_EXIT_TIME", state_transition_exit_time),
        (
            "STATE_TRANSITION_RANDOM_WEIGHT",
            state_transition_random_weight,
        ),
        ("STATE_MACHINE_INPUT_ID", state_machine_input_id),
        ("STATE_MACHINE_EVENT_ID", state_machine_event_id),
        ("STATE_MACHINE_FIRE_OCCURS", state_machine_fire_occurs),
        ("VIEW_MODEL_COMPONENT_NAME", view_model_component_name),
        (
            "VIEW_MODEL_INSTANCE_VIEW_MODEL_ID",
            view_model_instance_view_model_id,
        ),
        (
            "VIEW_MODEL_INSTANCE_VALUE_PROPERTY_ID",
            view_model_instance_value_property_id,
        ),
        (
            "VIEW_MODEL_INSTANCE_NUMBER_VALUE",
            view_model_instance_number_value,
        ),
        ("ARTBOARD_VIEW_MODEL_ID", artboard_view_model_id),
        ("DATA_BIND_PROPERTY_KEY", data_bind_property_key),
        ("DATA_BIND_FLAGS", data_bind_flags),
        ("DATA_BIND_SOURCE_PATH", data_bind_source_path),
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
        if matches!(
            object.rust_name,
            "FontAsset" | "ImageAsset" | "ScriptAsset" | "ShaderAsset"
        ) {
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
            "fn prop_{function_name}_apply(\n    record: &mut RecordSpec,\n    value: {},\n) -> std::result::Result<(), EditReason> {{",
            rust_type(resolved.runtime_type),
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
        writeln!(
            output,
            "    let RecordSpec::Visual {{ node, .. }} = record else {{\n        return Err(EditReason::RecordPropertyOwnerMismatch {{ property: {:?}, actual: record.kind() }});\n    }};",
            prop.schema_name,
        )
        .expect("write generated source");
        output.push_str("    match node {\n");
        for (object, field) in &compatible {
            writeln!(
                output,
                "        NodeSpec::{}(spec) => {{ spec.{} = value; Ok(()) }},",
                object.rust_name, field.rust_name
            )
            .expect("write generated source");
        }
        writeln!(
            output,
            "        other => Err(EditReason::PropertyOwnerMismatch {{ property: {:?}, actual: other.kind() }}),\n    }}\n}}\n",
            prop.schema_name,
        )
        .expect("write generated source");
    }

    output.push_str(
        "fn prop_nonvisual_is_available_on(_: NodeKind) -> bool { false }\n\n\
         fn prop_animation_fps_apply(record: &mut RecordSpec, value: u32) -> std::result::Result<(), EditReason> {\n\
             match record {\n\
                 RecordSpec::Animation(AnimationRecordSpec::LinearAnimation(spec)) => { spec.fps = value; Ok(()) },\n\
                 _ => Err(EditReason::RecordPropertyOwnerMismatch { property: \"fps\", actual: record.kind() }),\n\
             }\n\
         }\n\n\
         fn prop_animation_duration_apply(record: &mut RecordSpec, value: u32) -> std::result::Result<(), EditReason> {\n\
             match record {\n\
                 RecordSpec::Animation(AnimationRecordSpec::LinearAnimation(spec)) => { spec.duration = value; Ok(()) },\n\
                 _ => Err(EditReason::RecordPropertyOwnerMismatch { property: \"duration\", actual: record.kind() }),\n\
             }\n\
         }\n\n\
         fn prop_key_frame_double_value_apply(record: &mut RecordSpec, value: f32) -> std::result::Result<(), EditReason> {\n\
             if !value.is_finite() { return Err(EditReason::NonFiniteProperty { property: \"key_frame_value\" }); }\n\
             match record {\n\
                 RecordSpec::Animation(AnimationRecordSpec::KeyFrameDouble { value: stored, .. }) => { *stored = value; Ok(()) },\n\
                 _ => Err(EditReason::RecordPropertyOwnerMismatch { property: \"value\", actual: record.kind() }),\n\
             }\n\
         }\n\n",
    );

    output.push_str(
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub enum PropValueKind {\n\
             Double,\n\
             Color,\n\
             Uint,\n\
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
    for (name, ty, key, schema_name, value_kind, declared_owner, apply, setter, reader) in [
        (
            "ANIMATION_FPS",
            "u32",
            animation_fps.key.int,
            "fps",
            "Uint",
            "LinearAnimation",
            "prop_animation_fps_apply",
            "set_runtime_color",
            "read_runtime_color",
        ),
        (
            "ANIMATION_DURATION",
            "u32",
            animation_duration.key.int,
            "duration",
            "Uint",
            "LinearAnimation",
            "prop_animation_duration_apply",
            "set_runtime_color",
            "read_runtime_color",
        ),
        (
            "KEY_FRAME_DOUBLE_VALUE",
            "f32",
            key_frame_double_value.key.int,
            "value",
            "Double",
            "KeyFrameDouble",
            "prop_key_frame_double_value_apply",
            "set_runtime_double",
            "read_runtime_double",
        ),
    ] {
        writeln!(
            output,
            "    pub const {name}: Prop<{ty}> = Prop {{\n        key: {key},\n        schema_name: {schema_name:?},\n        value_kind: PropValueKind::{value_kind},\n        declared_owner: {declared_owner:?},\n        is_available_on: super::prop_nonvisual_is_available_on,\n        apply_to_definition: super::{apply},\n        apply_to_runtime: {setter},\n        read_from_runtime: {reader},\n        marker: PhantomData,\n    }};",
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
        ("ScriptedDrawable", "script") => "ScriptAssetId",
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
