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
];

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
    let mut output =
        String::from("// @generated by crates/nuxie/build.rs from nuxie-schema; do not edit.\n\n");

    writeln!(
        output,
        "pub(super) const TYPE_BACKBOARD: u16 = {};",
        backboard.type_key.int
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
        ("PATH_WIDTH", path_width),
        ("PATH_HEIGHT", path_height),
        ("FILL_RULE", fill_rule),
        ("COLOR_VALUE", color_value),
    ] {
        writeln!(
            output,
            "pub(super) const PROPERTY_{name}: u16 = {};",
            property.key.int
        )
        .expect("write generated source");
    }

    output.push('\n');
    for object in OBJECTS {
        writeln!(output, "#[derive(Debug, Clone, PartialEq)]").expect("write generated source");
        writeln!(output, "pub struct {}Spec {{", object.rust_name).expect("write generated source");
        for field in object.fields {
            let property = resolve_property(object.schema_name, *field);
            writeln!(
                output,
                "    pub {}: {},",
                field.rust_name,
                rust_type(property.runtime_type)
            )
            .expect("write generated source");
        }
        output.push_str("}\n\n");
    }

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
        other => panic!("unsupported public Scene spec field kind {other:?}"),
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
