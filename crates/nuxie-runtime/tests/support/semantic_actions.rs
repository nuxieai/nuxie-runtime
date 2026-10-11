// A file-authored semantic parent and child, with a typed native Tap listener.
// Slots: artboard0, parent shape1/rectangle2/semantics3,
// child shape4/rectangle5/semantics6. No fonts or host-created semantic nodes.
fn var(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 127) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 128;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}
fn key(kind: &str, name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(kind).unwrap();
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            nuxie_schema::definition_by_name(ancestor)
                .unwrap()
                .properties
                .iter()
        }))
        .find(|property| property.name == name)
        .unwrap()
        .key
        .int
}
fn object(bytes: &mut Vec<u8>, kind: &str, body: impl FnOnce(&mut Vec<u8>)) {
    var(
        bytes,
        u64::from(nuxie_schema::definition_by_name(kind).unwrap().type_key.int),
    );
    body(bytes);
    var(bytes, 0);
}
fn uint(bytes: &mut Vec<u8>, kind: &str, name: &str, value: u64) {
    var(bytes, u64::from(key(kind, name)));
    var(bytes, value);
}
fn float(bytes: &mut Vec<u8>, kind: &str, name: &str, value: f32) {
    var(bytes, u64::from(key(kind, name)));
    bytes.extend_from_slice(&value.to_le_bytes());
}
fn string(bytes: &mut Vec<u8>, kind: &str, name: &str, value: &str) {
    var(bytes, u64::from(key(kind, name)));
    var(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}
pub fn authored_semantic_tree(absorbed: bool) -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 9_641, 0] {
        var(&mut bytes, value);
    }
    object(&mut bytes, "Backboard", |_| {});
    object(&mut bytes, "Artboard", |b| {
        float(b, "Artboard", "width", 240.0);
        float(b, "Artboard", "height", 100.0);
    });
    object(&mut bytes, "Shape", |b| uint(b, "Component", "parentId", 0));
    object(&mut bytes, "Rectangle", |b| {
        uint(b, "Component", "parentId", 1);
        float(b, "ParametricPath", "width", 200.0);
        float(b, "ParametricPath", "height", 80.0);
    });
    object(&mut bytes, "SemanticData", |b| {
        uint(b, "Component", "parentId", 1);
        uint(b, "SemanticData", "role", if absorbed { 1 } else { 9 }); // Button / Group.
        string(
            b,
            "SemanticData",
            "label",
            if absorbed { "" } else { "Group" },
        );
        uint(b, "SemanticData", "traitFlags", 32); // Enablable.
        uint(
            b,
            "SemanticData",
            "stateFlags",
            if absorbed { 0 } else { 64 },
        ); // Disabled.
    });
    object(&mut bytes, "Shape", |b| uint(b, "Component", "parentId", 1));
    object(&mut bytes, "Rectangle", |b| {
        uint(b, "Component", "parentId", 4);
        float(b, "ParametricPath", "width", 40.0);
        float(b, "ParametricPath", "height", 20.0);
    });
    object(&mut bytes, "SemanticData", |b| {
        uint(b, "Component", "parentId", 4);
        uint(b, "SemanticData", "role", if absorbed { 7 } else { 1 }); // Text / Button.
        string(b, "SemanticData", "label", "Action");
    });
    object(&mut bytes, "StateMachine", |b| {
        string(b, "StateMachine", "name", "Main")
    });
    object(&mut bytes, "StateMachineListener", |b| {
        uint(b, "StateMachineListener", "targetId", 4)
    });
    object(&mut bytes, "ListenerInputTypeSemantic", |b| {
        uint(b, "ListenerInputType", "listenerTypeValue", 16)
    });
    bytes
}
