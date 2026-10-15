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

fn video(bytes: &mut Vec<u8>, readiness: u32) {
    var(bytes, 60001);
    uint(bytes, "Component", "parentId", 0);
    var(bytes, 206);
    var(bytes, 0);
    var(bytes, 60011);
    var(bytes, readiness.into());
    var(bytes, 0);
}
fn asset(bytes: &mut Vec<u8>) {
    var(bytes, 60000);
    var(bytes, 60000);
    let name = b"assets/greeting.mp4";
    var(bytes, name.len() as u64);
    bytes.extend(name);
    for (key, value) in [(208, 64.0_f32), (207, 32.0)] {
        var(bytes, key);
        bytes.extend(value.to_le_bytes());
    }
    var(bytes, 0);
}
fn binding(bytes: &mut Vec<u8>, key: u64, path: &[u8], flags: u64) {
    object(bytes, "DataBindContext", |b| {
        uint(b, "DataBind", "propertyKey", key);
        blob(b, "DataBindContext", "sourcePathIds", path);
        uint(b, "DataBind", "flags", flags);
    });
}
pub fn fixture(readiness: u32, timeline: bool) -> Vec<u8> {
    fixture_with_key(readiness, timeline.then_some(60006))
}
pub fn bool_timeline() -> Vec<u8> {
    fixture_with_key(0, Some(60005))
}
fn fixture_with_key(readiness: u32, timeline_key: Option<u64>) -> Vec<u8> {
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 3593, 0] {
        var(&mut b, n);
    }
    object(&mut b, "Backboard", |_| {});
    asset(&mut b);
    object(&mut b, "ViewModel", |b| {
        string(b, "ViewModel", "name", "VideoValues")
    });
    let fields = [
        ("Boolean", "muted", 0.0),
        ("Number", "volume", 1.0),
        ("Boolean", "paused", 1.0),
        ("Number", "seek", 0.0),
        ("Number", "position", 0.0),
        ("Number", "duration", 0.0),
        ("Boolean", "hostPaused", 1.0),
        ("Boolean", "ended", 0.0),
        ("Number", "state", 0.0),
        ("Number", "effectiveVolume", 0.0),
    ];
    for (kind, name, _) in fields {
        let kind = format!("ViewModelProperty{kind}");
        object(&mut b, &kind, |b| string(b, &kind, "name", name));
    }
    object(&mut b, "ViewModelInstance", |b| {
        uint(b, "ViewModelInstance", "viewModelId", 0)
    });
    for (i, (kind, _, value)) in fields.iter().enumerate() {
        let owner = format!("ViewModelInstance{kind}");
        object(&mut b, &owner, |b| {
            uint(b, &owner, "viewModelPropertyId", i as u64);
            if *kind == "Number" {
                float(b, &owner, "propertyValue", *value);
            } else {
                uint(b, &owner, "propertyValue", *value as u64);
            }
        });
    }
    object(&mut b, "Artboard", |b| {
        uint(b, "Artboard", "viewModelId", 0);
        float(b, "Artboard", "width", 64.0);
        float(b, "Artboard", "height", 32.0);
    });
    video(&mut b, readiness);
    for (i, key) in [
        60005, 60006, 60016, 60017, 60017, 60018, 60016, 60019, 60020, 60006,
    ]
    .iter()
    .enumerate()
    {
        if timeline_key == Some(*key) && i < 4 {
            continue;
        }
        binding(
            &mut b,
            *key,
            &[0, i as u8],
            if i == 2 { 2 } else { u64::from(i >= 4) },
        );
    }
    object(&mut b, "StateMachine", |b| {
        string(b, "StateMachine", "name", "Main")
    });
    if let Some(timeline_key) = timeline_key {
        object(&mut b, "LinearAnimation", |b| {
            string(b, "LinearAnimation", "name", "Volume");
            uint(b, "LinearAnimation", "fps", 60);
            uint(b, "LinearAnimation", "duration", 60);
        });
        object(&mut b, "KeyedObject", |b| {
            uint(b, "KeyedObject", "objectId", 1)
        });
        object(&mut b, "KeyedProperty", |b| {
            uint(b, "KeyedProperty", "propertyKey", timeline_key)
        });
        for (frame, value, interpolation) in [(0, 0.0, 1), (60, 1.0, 0)] {
            if timeline_key == 60005 {
                object(&mut b, "KeyFrameBool", |b| {
                    uint(b, "KeyFrameBool", "frame", frame);
                    uint(b, "KeyFrameBool", "value", value as u64);
                });
            } else {
                object(&mut b, "KeyFrameDouble", |b| {
                    uint(b, "KeyFrameDouble", "frame", frame);
                    uint(b, "KeyFrameDouble", "interpolationType", interpolation);
                    float(b, "KeyFrameDouble", "value", value);
                });
            }
        }
    }
    b
}

pub fn copies_and_rows() -> Vec<u8> {
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 3593, 0] {
        var(&mut b, n);
    }
    object(&mut b, "Backboard", |_| {});
    asset(&mut b);
    object(&mut b, "ViewModel", |b| {
        string(b, "ViewModel", "name", "Flags");
        uint(b, "ViewModel", "viewModelType", 2);
    });
    object(&mut b, "ViewModelPropertyBoolean", |b| {
        string(b, "ViewModelPropertyBoolean", "name", "muted")
    });
    object(&mut b, "ViewModelInstance", |b| {
        uint(b, "ViewModelInstance", "viewModelId", 0)
    });
    object(&mut b, "ViewModelInstanceBoolean", |b| {
        uint(b, "ViewModelInstanceBoolean", "viewModelPropertyId", 0)
    });
    object(&mut b, "ViewModel", |b| {
        string(b, "ViewModel", "name", "Root")
    });
    object(&mut b, "ViewModelPropertyList", |b| {
        string(b, "ViewModelPropertyList", "name", "rows")
    });
    object(&mut b, "ViewModelInstance", |b| {
        uint(b, "ViewModelInstance", "viewModelId", 1)
    });
    object(&mut b, "ViewModelInstanceList", |b| {
        uint(b, "ViewModelInstanceList", "viewModelPropertyId", 0)
    });
    object(&mut b, "ViewModelInstanceListItem", |b| {
        uint(b, "ViewModelInstanceListItem", "viewModelId", 2);
        uint(b, "ViewModelInstanceListItem", "viewModelInstanceId", 0);
    });
    object(&mut b, "ViewModel", |b| {
        string(b, "ViewModel", "name", "Row")
    });
    object(&mut b, "ViewModelInstance", |b| {
        uint(b, "ViewModelInstance", "viewModelId", 2)
    });
    object(&mut b, "Artboard", |b| {
        uint(b, "Artboard", "viewModelId", 2)
    });
    video(&mut b, 0);
    binding(&mut b, 60005, &[0, 0], 0);
    object(&mut b, "Artboard", |b| {
        uint(b, "Artboard", "viewModelId", 1)
    });
    video(&mut b, 0);
    binding(&mut b, 60005, &[0, 0], 0);
    object(&mut b, "NestedArtboard", |b| {
        uint(b, "Component", "parentId", 0);
        uint(b, "NestedArtboard", "artboardId", 0);
    });
    object(&mut b, "ArtboardComponentList", |b| {
        uint(b, "Component", "parentId", 0)
    });
    binding(
        &mut b,
        key("ArtboardComponentList", "listSource").into(),
        &[1, 0],
        0,
    );
    object(&mut b, "StateMachine", |b| {
        string(b, "StateMachine", "name", "Main")
    });
    b
}
