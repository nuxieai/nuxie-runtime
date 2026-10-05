// Independent binary fixture: a global flag is read by a screen, copy and row.
fn var(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 127) as u8;
        value >>= 7;
        bytes.push(byte | if value == 0 { 0 } else { 128 });
        if value == 0 {
            break;
        }
    }
}
fn key(kind: &str, name: &str) -> u16 {
    let d = nuxie_schema::definition_by_name(kind).unwrap();
    std::iter::once(d.name)
        .chain(d.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|d| d.properties)
        .find(|p| p.name == name)
        .unwrap()
        .key
        .int
}
fn object(bytes: &mut Vec<u8>, kind: &str, body: impl FnOnce(&mut Vec<u8>)) {
    var(
        bytes,
        nuxie_schema::definition_by_name(kind)
            .unwrap()
            .type_key
            .int
            .into(),
    );
    body(bytes);
    var(bytes, 0);
}
fn uint(bytes: &mut Vec<u8>, kind: &str, name: &str, value: u64) {
    var(bytes, key(kind, name).into());
    var(bytes, value);
}
fn string(bytes: &mut Vec<u8>, kind: &str, name: &str, value: &str) {
    blob(bytes, kind, name, value.as_bytes());
}
fn blob(bytes: &mut Vec<u8>, kind: &str, name: &str, value: &[u8]) {
    var(bytes, key(kind, name).into());
    var(bytes, value.len() as u64);
    bytes.extend(value);
}
fn float(bytes: &mut Vec<u8>, kind: &str, name: &str, value: f32) {
    var(bytes, key(kind, name).into());
    bytes.extend(value.to_le_bytes());
}
fn binding(bytes: &mut Vec<u8>, kind: &str, name: &str, path: &[u8], converter: bool) {
    object(bytes, "DataBindContext", |b| {
        uint(b, "DataBind", "propertyKey", key(kind, name).into());
        blob(b, "DataBindContext", "sourcePathIds", path);
        if converter {
            uint(b, "DataBind", "converterId", 0);
        }
    });
}
fn label(bytes: &mut Vec<u8>) {
    object(bytes, "Shape", |b| {
        uint(b, "Component", "parentId", 0);
    }); // 1
    object(bytes, "Rectangle", |b| {
        // 2
        uint(b, "Component", "parentId", 1);
        float(b, "ParametricPath", "width", 50.0);
        float(b, "ParametricPath", "height", 20.0);
    });
    object(bytes, "SemanticData", |b| {
        // 3
        uint(b, "Component", "parentId", 1);
        uint(b, "SemanticData", "role", 7);
        string(b, "SemanticData", "label", "unbound");
    });
    binding(bytes, "SemanticData", "label", &[0, 0], true);
}
pub fn fixture() -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for n in [7, 0, 3593, 0] {
        var(&mut bytes, n);
    }
    object(&mut bytes, "Backboard", |_| {});
    object(&mut bytes, "DataConverterToString", |_| {});
    object(&mut bytes, "ViewModel", |b| {
        string(b, "ViewModel", "name", "Flags");
        uint(b, "ViewModel", "viewModelType", 2);
    });
    object(&mut bytes, "ViewModelPropertyBoolean", |b| {
        string(b, "ViewModelPropertyBoolean", "name", "enabled")
    });
    object(&mut bytes, "ViewModelInstance", |b| {
        uint(b, "ViewModelInstance", "viewModelId", 0);
        string(b, "ViewModelInstance", "name", "default");
    });
    object(&mut bytes, "ViewModelInstanceBoolean", |b| {
        uint(b, "ViewModelInstanceBoolean", "viewModelPropertyId", 0);
    });
    object(&mut bytes, "ViewModel", |b| {
        string(b, "ViewModel", "name", "Root")
    });
    object(&mut bytes, "ViewModelPropertyList", |b| {
        string(b, "ViewModelPropertyList", "name", "rows")
    });
    object(&mut bytes, "ViewModelInstance", |b| {
        uint(b, "ViewModelInstance", "viewModelId", 1);
    });
    object(&mut bytes, "ViewModelInstanceList", |b| {
        uint(b, "ViewModelInstanceList", "viewModelPropertyId", 0);
    });
    object(&mut bytes, "ViewModelInstanceListItem", |b| {
        uint(b, "ViewModelInstanceListItem", "viewModelId", 2);
        uint(b, "ViewModelInstanceListItem", "viewModelInstanceId", 0);
    });
    object(&mut bytes, "ViewModel", |b| {
        string(b, "ViewModel", "name", "Row")
    });
    object(&mut bytes, "ViewModelInstance", |b| {
        uint(b, "ViewModelInstance", "viewModelId", 2);
    });
    object(&mut bytes, "Artboard", |b| {
        uint(b, "Artboard", "viewModelId", 2);
        float(b, "Artboard", "width", 60.0);
        float(b, "Artboard", "height", 30.0);
    });
    label(&mut bytes);
    object(&mut bytes, "Artboard", |b| {
        uint(b, "Artboard", "viewModelId", 1);
        float(b, "Artboard", "width", 400.0);
        float(b, "Artboard", "height", 100.0);
    });
    label(&mut bytes);
    object(&mut bytes, "NestedArtboard", |b| {
        // 4
        uint(b, "Component", "parentId", 0);
        uint(b, "NestedArtboard", "artboardId", 0);
        float(b, "Node", "x", 100.0);
    });
    object(&mut bytes, "ArtboardComponentList", |b| {
        // 5
        uint(b, "Component", "parentId", 0);
        float(b, "Node", "x", 200.0);
    });
    binding(
        &mut bytes,
        "ArtboardComponentList",
        "listSource",
        &[1, 0],
        false,
    );
    object(&mut bytes, "StateMachine", |b| {
        string(b, "StateMachine", "name", "Main")
    });
    object(&mut bytes, "LinearAnimation", |b| {
        string(b, "LinearAnimation", "name", "Timeline");
    });
    bytes
}
