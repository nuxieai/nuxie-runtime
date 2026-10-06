//! Repository-owned, deterministic solid-fill fixture for live pixel probes.

pub const ARTBOARD_SIZE: u32 = 64;
pub const FILL_RGBA: [u8; 4] = [0x33, 0x66, 0xaa, 0xff];

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
    let definition = nuxie_schema::definition_by_name(type_name).expect("solid-fill fixture type");
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            nuxie_schema::definition_by_name(ancestor)
                .expect("solid-fill fixture ancestor")
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .expect("solid-fill fixture property")
        .key
        .int
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, body: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(
            nuxie_schema::definition_by_name(type_name)
                .expect("solid-fill fixture type")
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

fn push_color(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u32) {
    push_var_uint(bytes, u64::from(property_key(type_name, property_name)));
    bytes.extend_from_slice(&value.to_le_bytes());
}

/// A 64x64 artboard covered by one opaque #3366AA rectangle.
pub fn solid_fill_artboard() -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 9_641, 0] {
        push_var_uint(&mut bytes, value);
    }
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", ARTBOARD_SIZE as f32);
        push_f32(bytes, "Artboard", "height", ARTBOARD_SIZE as f32);
    });
    push_object(&mut bytes, "Shape", |bytes| {
        push_uint(bytes, "Node", "parentId", 0);
    });
    push_object(&mut bytes, "Fill", |bytes| {
        push_uint(bytes, "Component", "parentId", 1);
    });
    push_object(&mut bytes, "SolidColor", |bytes| {
        push_uint(bytes, "Component", "parentId", 2);
        push_color(bytes, "SolidColor", "colorValue", 0xff33_66aa);
    });
    push_object(&mut bytes, "Rectangle", |bytes| {
        push_uint(bytes, "Node", "parentId", 1);
        push_f32(bytes, "Node", "x", ARTBOARD_SIZE as f32 / 2.0);
        push_f32(bytes, "Node", "y", ARTBOARD_SIZE as f32 / 2.0);
        push_f32(bytes, "ParametricPath", "width", ARTBOARD_SIZE as f32);
        push_f32(bytes, "ParametricPath", "height", ARTBOARD_SIZE as f32);
    });
    bytes
}
