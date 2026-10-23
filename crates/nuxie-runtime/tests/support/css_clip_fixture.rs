//! The same positioned text participant for interaction and host projections.

fn uint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        bytes.push(byte | if value == 0 { 0 } else { 0x80 });
        if value == 0 {
            break;
        }
    }
}

fn key(owner: &str, name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(owner).unwrap();
    std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|property| property.name == name)
        .unwrap()
        .key
        .int
}

fn object(bytes: &mut Vec<u8>, kind: &str, props: impl FnOnce(&mut Vec<u8>)) {
    uint(
        bytes,
        nuxie_schema::definition_by_name(kind)
            .unwrap()
            .type_key
            .int
            .into(),
    );
    props(bytes);
    uint(bytes, 0);
}

fn number(bytes: &mut Vec<u8>, owner: &str, name: &str, value: u64) {
    uint(bytes, key(owner, name).into());
    uint(bytes, value);
}

fn float(bytes: &mut Vec<u8>, owner: &str, name: &str, value: f32) {
    uint(bytes, key(owner, name).into());
    bytes.extend(value.to_le_bytes());
}

fn string(bytes: &mut Vec<u8>, owner: &str, name: &str, value: &str) {
    uint(bytes, key(owner, name).into());
    uint(bytes, value.len() as u64);
    bytes.extend(value.as_bytes());
}

fn css(bytes: &mut Vec<u8>, parent: u64, enums: &[(u16, u64)], floats: &[(u16, f32)]) {
    uint(bytes, 61000);
    number(bytes, "Component", "parentId", parent);
    for &(key, value) in enums {
        uint(bytes, key.into());
        uint(bytes, value);
    }
    for &(key, value) in floats {
        uint(bytes, key.into());
        bytes.extend(value.to_le_bytes());
    }
    uint(bytes, 0);
}

/// A text participant starts at x=80 inside a 100-point clipping box.
/// A static box lets it escape; a positioned box is its containing block.
pub fn absolute_text(parent_position: u64) -> Vec<u8> {
    absolute_text_scene(parent_position, true, 240.0, 100.0)
}

/// Vary the root mode and frame independently from the CSS overflow box.
pub fn absolute_text_scene(
    parent_position: u64,
    root_css: bool,
    root_width: f32,
    clip_width: f32,
) -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 0, 0] {
        uint(&mut bytes, value);
    }
    object(&mut bytes, "Backboard", |_| {});
    object(&mut bytes, "FontAsset", |bytes| {
        number(bytes, "FileAsset", "assetId", 1);
    });
    object(&mut bytes, "FileAssetContents", |bytes| {
        let font = include_bytes!("../../../../fixtures/fonts/roboto-input.ttf");
        uint(bytes, key("FileAssetContents", "bytes").into());
        uint(bytes, font.len() as u64);
        bytes.extend(font);
    });
    object(&mut bytes, "Artboard", |bytes| {
        float(bytes, "LayoutComponent", "width", root_width);
        float(bytes, "LayoutComponent", "height", 120.0);
        number(bytes, "LayoutComponent", "styleId", 12);
    });
    if root_css {
        css(&mut bytes, 0, &[(61003, 2), (61004, 2)], &[]);
    } else {
        // Preserve object ids without putting the root in CSS mode.
        object(&mut bytes, "Node", |bytes| {
            number(bytes, "Component", "parentId", 0)
        });
    }
    object(&mut bytes, "LayoutComponent", |bytes| {
        number(bytes, "Component", "parentId", 0);
        string(bytes, "Component", "name", "Clip");
        number(bytes, "LayoutComponent", "styleId", 13);
    });
    css(
        &mut bytes,
        2,
        &[
            (61002, parent_position),
            (61003, 2),
            (61004, 2),
            (61021, 1),
            (61024, 1),
        ],
        &[(61022, clip_width), (61025, 60.0), (61009, 0.0)],
    );
    object(&mut bytes, "Text", |bytes| {
        number(bytes, "Component", "parentId", 2);
        string(bytes, "Component", "name", "Escaped text");
    });
    object(&mut bytes, "LayoutParticipant", |bytes| {
        number(bytes, "Component", "parentId", 4);
    });
    css(
        &mut bytes,
        4,
        &[(61002, 2), (61060, 1), (61051, 1), (61021, 1), (61024, 1)],
        &[(61061, 80.0), (61052, 10.0), (61022, 160.0), (61025, 24.0)],
    );
    object(&mut bytes, "TextValueRun", |bytes| {
        number(bytes, "Component", "parentId", 4);
        number(bytes, "TextValueRun", "styleId", 8);
        string(bytes, "TextValueRun", "text", "MMMMMMMMMMMM");
    });
    object(&mut bytes, "TextStylePaint", |bytes| {
        number(bytes, "Component", "parentId", 4);
        number(bytes, "TextStyle", "fontAssetId", 0);
        float(bytes, "TextStyle", "fontSize", 20.0);
    });
    object(&mut bytes, "Fill", |bytes| {
        number(bytes, "Component", "parentId", 8);
    });
    object(&mut bytes, "SolidColor", |bytes| {
        number(bytes, "Component", "parentId", 9);
        uint(bytes, key("SolidColor", "colorValue").into());
        bytes.extend(0xff44_5566u32.to_le_bytes());
    });
    object(&mut bytes, "Event", |bytes| {
        number(bytes, "Component", "parentId", 0);
        string(bytes, "Component", "name", "text clicked");
    });
    for parent in [0, 2] {
        object(&mut bytes, "LayoutComponentStyle", |bytes| {
            number(bytes, "Component", "parentId", parent);
        });
    }
    object(&mut bytes, "StateMachine", |_| {});
    object(&mut bytes, "StateMachineListener", |bytes| {
        number(bytes, "StateMachineListener", "targetId", 4);
    });
    object(&mut bytes, "ListenerInputType", |bytes| {
        number(bytes, "ListenerInputType", "listenerTypeValue", 6);
    });
    object(&mut bytes, "ListenerFireEvent", |bytes| {
        number(bytes, "ListenerFireEvent", "eventId", 11);
    });
    object(&mut bytes, "StateMachineLayer", |_| {});
    for kind in ["EntryState", "AnyState", "ExitState"] {
        object(&mut bytes, kind, |_| {});
    }
    bytes
}
