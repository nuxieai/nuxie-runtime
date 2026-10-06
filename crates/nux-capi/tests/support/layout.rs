// Authored layout fixture with a percentage-width semantic child.
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
    let definition = nuxie_schema::definition_by_name(type_name).expect("layout fixture type");
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            nuxie_schema::definition_by_name(ancestor)
                .expect("layout fixture ancestor")
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .expect("layout fixture property")
        .key
        .int
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, body: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(
            nuxie_schema::definition_by_name(type_name)
                .expect("layout fixture type")
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

pub fn layout_artboard() -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 9_641, 0] {
        push_var_uint(&mut bytes, value);
    }
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 393.0);
        push_f32(bytes, "Artboard", "height", 852.0);
        push_uint(bytes, "LayoutComponent", "styleId", 1);
    });
    push_object(&mut bytes, "LayoutComponentStyle", |bytes| {
        push_f32(bytes, "LayoutComponentStyle", "paddingTop", 16.0);
        push_f32(bytes, "LayoutComponentStyle", "paddingBottom", 16.0);
        push_uint(bytes, "LayoutComponentStyle", "paddingTopUnitsValue", 1);
        push_uint(bytes, "LayoutComponentStyle", "paddingBottomUnitsValue", 1);
    });
    push_object(&mut bytes, "LayoutComponent", |bytes| {
        push_uint(bytes, "Component", "parentId", 0);
        push_f32(bytes, "LayoutComponent", "width", 100.0);
        push_f32(bytes, "LayoutComponent", "height", 120.0);
        push_uint(bytes, "LayoutComponent", "styleId", 3);
    });
    push_object(&mut bytes, "LayoutComponentStyle", |bytes| {
        push_uint(bytes, "LayoutComponentStyle", "widthUnitsValue", 2);
    });
    push_object(&mut bytes, "SemanticData", |bytes| {
        push_uint(bytes, "Component", "parentId", 2);
        push_uint(bytes, "SemanticData", "role", 1);
        push_string(bytes, "SemanticData", "label", "Full width child");
    });
    bytes
}
