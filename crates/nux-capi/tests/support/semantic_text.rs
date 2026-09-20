// Authored binary fixture for text-run/semantic ownership; no platform font is required.
fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition =
        nuxie_schema::definition_by_name(type_name).expect("semantic-text fixture type");
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            nuxie_schema::definition_by_name(ancestor)
                .expect("semantic-text fixture ancestor")
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .expect("semantic-text fixture property")
        .key
        .int
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, body: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(
            nuxie_schema::definition_by_name(type_name)
                .expect("semantic-text fixture type")
                .type_key
                .int,
        ),
    );
    body(bytes);
    push_var_uint(bytes, 0);
}

fn push_uint(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u64) {
    push_var_uint(bytes, u64::from(property_key(type_name, property_name)));
    push_var_uint(bytes, value);
}

fn push_f32(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: f32) {
    push_var_uint(bytes, u64::from(property_key(type_name, property_name)));
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_string(bytes: &mut Vec<u8>, kind: &str, property: &str, value: &str) {
    push_var_uint(bytes, u64::from(property_key(kind, property)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

pub fn semantic_text_artboard() -> Vec<u8> {
    text_artboard(false)
}

pub fn compound_semantic_text_artboard() -> Vec<u8> {
    text_artboard(true)
}

pub fn with_string_properties(mut bytes: Vec<u8>, names: &[&str]) -> Vec<u8> {
    for name in names {
        push_object(&mut bytes, "CustomPropertyString", |bytes| {
            push_uint(bytes, "Component", "parentId", 0);
            push_string(bytes, "Component", "name", name);
            push_string(
                bytes,
                "CustomPropertyString",
                "propertyValue",
                "editable value",
            );
        });
    }
    bytes
}

pub fn repeated_nonvisual_fields() -> Vec<u8> {
    repeated_fields(None, false, 1)
}

pub fn repeated_native_input_fields(obscured: bool) -> Vec<u8> {
    repeated_fields(Some(obscured), false, 1)
}

pub fn transformed_native_input_fields(obscured: bool) -> Vec<u8> {
    repeated_fields(Some(obscured), true, 1)
}

pub fn repeated_pair_native_input_fields(obscured: bool) -> Vec<u8> {
    repeated_fields(Some(obscured), false, 2)
}

pub fn shaped_native_input(obscured: bool, text: &str) -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 9_641, 0] {
        push_var_uint(&mut bytes, value);
    }
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "FontAsset", |bytes| {
        push_uint(bytes, "FontAsset", "assetId", 7);
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        let font = include_bytes!("../../../../fixtures/fonts/roboto-input.ttf");
        push_var_uint(bytes, u64::from(property_key("FileAssetContents", "bytes")));
        push_var_uint(bytes, font.len() as u64);
        bytes.extend_from_slice(font);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 240.0);
        push_f32(bytes, "Artboard", "height", 100.0);
    });
    push_object(&mut bytes, "Shape", |bytes| {
        push_uint(bytes, "Component", "parentId", 0);
        push_f32(bytes, "Node", "x", 12.0);
        push_f32(bytes, "Node", "y", 18.0);
    });
    push_object(&mut bytes, "Rectangle", |bytes| {
        push_uint(bytes, "Component", "parentId", 1);
        push_f32(bytes, "ParametricPath", "width", 200.0);
        push_f32(bytes, "ParametricPath", "height", 60.0);
    });
    push_object(&mut bytes, "SemanticData", |bytes| {
        push_uint(bytes, "Component", "parentId", 1);
        push_uint(bytes, "SemanticData", "role", 6);
    });
    push_object(&mut bytes, "TextInput", |bytes| {
        push_uint(bytes, "Component", "parentId", 1);
        push_string(bytes, "Component", "name", "editable");
        push_string(bytes, "TextInput", "text", text);
        push_uint(bytes, "TextInput", "multiline", 1);
        push_uint(bytes, "TextInput", "obscured", u64::from(obscured));
    });
    push_object(&mut bytes, "TextStylePaint", |bytes| {
        push_uint(bytes, "Component", "parentId", 4);
        // Style references the file asset index, not FileAsset.assetId.
        push_uint(bytes, "TextStyle", "fontAssetId", 0);
        push_f32(bytes, "TextStyle", "fontSize", 24.0);
    });
    bytes
}

fn repeated_fields(native_input: Option<bool>, transformed: bool, field_count: u64) -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 9_641, 0] {
        push_var_uint(&mut bytes, value);
    }
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 120.0);
        push_f32(bytes, "Artboard", "height", 60.0);
    });
    for field in 0..field_count {
        let shape_id = 1 + field * 4;
        push_object(&mut bytes, "Shape", |bytes| {
            push_uint(bytes, "Component", "parentId", 0);
            push_f32(bytes, "Node", "y", field as f32 * 50.0);
        });
        push_object(&mut bytes, "Rectangle", |bytes| {
            push_uint(bytes, "Component", "parentId", shape_id);
            push_f32(bytes, "ParametricPath", "width", 100.0);
            push_f32(bytes, "ParametricPath", "height", 40.0);
        });
        push_object(&mut bytes, "SemanticData", |bytes| {
            push_uint(bytes, "Component", "parentId", shape_id);
            push_uint(bytes, "SemanticData", "role", 6);
            push_string(bytes, "SemanticData", "label", "Field");
        });
        if let Some(obscured) = native_input {
            push_object(&mut bytes, "TextInput", |bytes| {
                push_uint(bytes, "Component", "parentId", shape_id);
                push_string(bytes, "Component", "name", "editable");
                if transformed {
                    push_f32(bytes, "Node", "x", 7.0);
                    push_f32(bytes, "Node", "y", 11.0);
                }
                push_string(bytes, "TextInput", "text", "editable value");
                push_uint(bytes, "TextInput", "obscured", u64::from(obscured));
            });
        } else {
            bytes = with_string_properties(bytes, &["editable"]);
        }
    }
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 400.0);
        push_f32(bytes, "Artboard", "height", 100.0);
    });
    for x in [60.0, 240.0] {
        push_object(&mut bytes, "NestedArtboard", |bytes| {
            push_uint(bytes, "Component", "parentId", 0);
            push_uint(bytes, "NestedArtboard", "artboardId", 0);
            push_f32(bytes, "Node", "x", x);
            push_f32(bytes, "Node", "y", 30.0);
            if transformed {
                push_f32(bytes, "Node", "rotation", std::f32::consts::FRAC_PI_2);
                push_f32(bytes, "Node", "scaleX", 2.0);
                push_f32(bytes, "Node", "scaleY", 3.0);
            }
        });
    }
    bytes
}

fn text_artboard(compound: bool) -> Vec<u8> {
    text_artboard_with_transform(compound, None)
}

pub fn transformed_compound_text_artboard(transform: [f32; 7]) -> Vec<u8> {
    text_artboard_with_transform(true, Some(transform))
}

fn text_artboard_with_transform(compound: bool, transform: Option<[f32; 7]>) -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 9_641, 0] {
        push_var_uint(&mut bytes, value);
    }
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 200.0);
        push_f32(bytes, "Artboard", "height", 100.0);
    });
    if compound {
        push_object(&mut bytes, "Shape", |bytes| {
            push_uint(bytes, "Component", "parentId", 0);
            if let Some([x, y, rotation, scale_x, scale_y, _, _]) = transform {
                for (property, value) in [
                    ("x", x),
                    ("y", y),
                    ("rotation", rotation),
                    ("scaleX", scale_x),
                    ("scaleY", scale_y),
                ] {
                    push_f32(bytes, "Node", property, value);
                }
            }
        });
        push_object(&mut bytes, "Rectangle", |bytes| {
            push_uint(bytes, "Component", "parentId", 1);
            push_f32(bytes, "ParametricPath", "width", 100.0);
            push_f32(bytes, "ParametricPath", "height", 40.0);
        });
    }
    let text_id = if compound { 3 } else { 1 };
    let style_id = text_id + 1;
    push_object(&mut bytes, "Text", |bytes| {
        push_uint(bytes, "Component", "parentId", if compound { 1 } else { 0 });
        if let Some([_, _, _, _, _, x, y]) = transform {
            push_f32(bytes, "Node", "x", x);
            push_f32(bytes, "Node", "y", y);
        }
        push_f32(bytes, "Text", "width", 100.0);
        push_f32(bytes, "Text", "height", 40.0);
    });
    push_object(&mut bytes, "TextStyle", |bytes| {
        push_uint(bytes, "Component", "parentId", text_id);
    });
    push_object(&mut bytes, "SemanticData", |bytes| {
        push_uint(bytes, "Component", "parentId", 1);
        push_uint(bytes, "SemanticData", "role", 7);
        push_string(bytes, "SemanticData", "label", "Name");
    });
    if compound {
        push_object(&mut bytes, "SemanticData", |bytes| {
            push_uint(bytes, "Component", "parentId", text_id);
            push_uint(bytes, "SemanticData", "role", 7);
            push_string(bytes, "SemanticData", "label", "Inner text");
        });
    }
    for name in ["field/name", "field/other"] {
        push_object(&mut bytes, "TextValueRun", |bytes| {
            push_uint(bytes, "Component", "parentId", text_id);
            push_uint(bytes, "TextValueRun", "styleId", style_id);
            push_string(bytes, "Component", "name", name);
            push_string(bytes, "TextValueRun", "text", "private field value");
        });
    }
    bytes
}

pub fn nested_layout_text_artboard() -> Vec<u8> {
    nested_layout_artboard(false)
}

pub fn nested_layout_native_input_artboard() -> Vec<u8> {
    nested_layout_artboard(true)
}

pub fn nested_layout_native_input_occurrence() -> Vec<u8> {
    let mut bytes = nested_layout_artboard(true);
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 1000.0);
        push_f32(bytes, "Artboard", "height", 1000.0);
    });
    push_object(&mut bytes, "NestedArtboard", |bytes| {
        push_uint(bytes, "Component", "parentId", 0);
        push_uint(bytes, "NestedArtboard", "artboardId", 0);
        push_f32(bytes, "Node", "x", 200.0);
        push_f32(bytes, "Node", "y", 150.0);
        push_f32(bytes, "Node", "scaleX", 0.5);
        push_f32(bytes, "Node", "scaleY", 0.5);
    });
    bytes
}

fn nested_layout_artboard(native: bool) -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 9_641, 0] {
        push_var_uint(&mut bytes, value);
    }
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 500.0);
        push_f32(bytes, "Artboard", "height", 500.0);
    });
    push_object(&mut bytes, "LayoutComponent", |bytes| {
        push_uint(bytes, "Component", "parentId", 0);
        push_f32(bytes, "LayoutComponent", "width", 300.0);
        push_f32(bytes, "LayoutComponent", "height", 200.0);
        push_uint(bytes, "LayoutComponent", "styleId", 2);
    });
    push_object(&mut bytes, "LayoutComponentStyle", |_| {});
    push_object(&mut bytes, "Node", |bytes| {
        push_uint(bytes, "Component", "parentId", 1);
        for (property, value) in [
            ("x", 24.0),
            ("y", 24.0),
            ("rotation", std::f32::consts::FRAC_PI_2),
            ("scaleX", 2.0),
            ("scaleY", 3.0),
        ] {
            push_f32(bytes, "Node", property, value);
        }
    });
    push_object(&mut bytes, "LayoutComponent", |bytes| {
        push_uint(bytes, "Component", "parentId", 3);
        push_f32(bytes, "LayoutComponent", "width", 100.0);
        push_f32(bytes, "LayoutComponent", "height", 40.0);
        push_uint(bytes, "LayoutComponent", "styleId", 5);
    });
    push_object(&mut bytes, "LayoutComponentStyle", |_| {});
    push_object(&mut bytes, if native { "TextInput" } else { "Text" }, |bytes| {
        push_uint(bytes, "Component", "parentId", 4);
        push_f32(bytes, "Node", "x", 7.0);
        push_f32(bytes, "Node", "y", 11.0);
        if native {
            push_string(bytes, "Component", "name", "editable");
        }
    });
    if native {
        push_object(&mut bytes, "SemanticData", |bytes| {
            push_uint(bytes, "Component", "parentId", 4);
            push_uint(bytes, "SemanticData", "role", 6);
        });
        return bytes;
    }
    push_object(&mut bytes, "TextStyle", |bytes| {
        push_uint(bytes, "Component", "parentId", 6);
    });
    push_object(&mut bytes, "TextValueRun", |bytes| {
        push_uint(bytes, "Component", "parentId", 6);
        push_uint(bytes, "TextValueRun", "styleId", 7);
        push_string(bytes, "Component", "name", "field/name");
        push_string(bytes, "TextValueRun", "text", "private value");
    });
    bytes
}
