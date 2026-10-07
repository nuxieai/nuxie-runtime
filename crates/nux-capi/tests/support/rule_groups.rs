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
    let definition = nuxie_schema::definition_by_name(type_name).unwrap();
    std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|property| property.name == property_name)
        .unwrap()
        .key
        .int
}

fn object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(
            nuxie_schema::definition_by_name(type_name)
                .unwrap()
                .type_key
                .int,
        ),
    );
    properties(bytes);
    push_var_uint(bytes, 0);
}

fn uint(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u64) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value);
}

fn string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

fn blob(bytes: &mut Vec<u8>, kind: &str, name: &str, value: &[u8]) {
    push_var_uint(bytes, u64::from(property_key(kind, name)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value);
}
fn float(bytes: &mut Vec<u8>, kind: &str, name: &str, value: f32) {
    push_var_uint(bytes, u64::from(property_key(kind, name)));
    bytes.extend_from_slice(&value.to_le_bytes());
}
#[derive(Clone, Copy)]
#[allow(
    dead_code,
    reason = "shared by unit and integration tests with different listener cases"
)]
pub enum Action {
    Number(f32),
    Text(&'static str),
    Marker(bool),
    Script,
}
fn binding(
    b: &mut Vec<u8>,
    kind: &str,
    property: &str,
    source: u8,
    converter: Option<u64>,
    flags: u64,
) {
    object(b, "DataBindContext", |b| {
        uint(
            b,
            "DataBind",
            "propertyKey",
            u64::from(property_key(kind, property)),
        );
        blob(b, "DataBindContext", "sourcePathIds", &[0, source]);
        if let Some(id) = converter {
            uint(b, "DataBind", "converterId", id);
        }
        uint(b, "DataBind", "flags", flags);
    });
}

pub fn fixture() -> Vec<u8> {
    let mut b = b"RIVE".to_vec();
    for value in [7, 0, 3593, 0] {
        push_var_uint(&mut b, value);
    }
    object(&mut b, "Backboard", |_| {});
    object(&mut b, "DataEnumCustom", |b| {
        string(b, "DataEnumCustom", "name", "Levels")
    });
    for value in ["beginner", "advanced"] {
        object(&mut b, "DataEnumValue", |b| {
            string(b, "DataEnumValue", "key", value);
            string(b, "DataEnumValue", "value", value);
        });
    }
    for (model, name) in [(0, "First"), (1, "Second")] {
        object(&mut b, "ViewModel", |b| {
            string(b, "ViewModel", "name", name)
        });
        for (kind, name) in [
            ("Number", "trip_days"),
            ("Boolean", "trip_days_set"),
            ("EnumCustom", "italian_level"),
            ("Boolean", "italian_level_set"),
            ("Boolean", "wants_reminder"),
            ("Boolean", "wants_reminder_set"),
            ("String", "email"),
            ("Boolean", "valid"),
            ("ViewModel", "errors"),
            ("Boolean", "saving"),
            ("Boolean", "saved"),
            ("String", "saveError"),
        ] {
            let kind = format!("ViewModelProperty{kind}");
            object(&mut b, &kind, |b| {
                string(b, &kind, "name", name);
                if name == "errors" {
                    uint(b, &kind, "viewModelReferenceId", 2);
                }
                if name == "italian_level" {
                    uint(b, &kind, "enumId", 0);
                }
            });
        }
        object(&mut b, "ViewModelInstance", |b| {
            uint(b, "ViewModelInstance", "viewModelId", model)
        });
        for (index, kind) in [
            "Number",
            "Boolean",
            "Enum",
            "Boolean",
            "Boolean",
            "Boolean",
            "String",
            "Boolean",
            "ViewModel",
            "Boolean",
            "Boolean",
            "String",
        ]
        .iter()
        .enumerate()
        {
            let kind = format!("ViewModelInstance{kind}");
            object(&mut b, &kind, |b| {
                uint(b, &kind, "viewModelPropertyId", index as u64)
            });
        }
    }
    object(&mut b, "ViewModel", |b| {
        string(b, "ViewModel", "name", "Errors")
    });
    for name in ["trip_days", "italian_level", "wants_reminder", "email"] {
        object(&mut b, "ViewModelPropertyList", |b| {
            string(b, "ViewModelPropertyList", "name", name)
        });
    }
    object(&mut b, "ViewModelInstance", |b| {
        uint(b, "ViewModelInstance", "viewModelId", 2)
    });
    for index in 0..4 {
        object(&mut b, "ViewModelInstanceList", |b| {
            uint(b, "ViewModelInstanceList", "viewModelPropertyId", index)
        });
    }
    object(&mut b, "ViewModel", |b| {
        string(b, "ViewModel", "name", "ErrorEntry")
    });
    for name in ["rule", "message"] {
        object(&mut b, "ViewModelPropertyString", |b| {
            string(b, "ViewModelPropertyString", "name", name)
        });
    }
    object(&mut b, "ViewModelInstance", |b| {
        uint(b, "ViewModelInstance", "viewModelId", 3)
    });
    for index in 0..2 {
        object(&mut b, "ViewModelInstanceString", |b| {
            uint(b, "ViewModelInstanceString", "viewModelPropertyId", index)
        });
    }
    object(&mut b, "Artboard", |b| {
        uint(b, "Artboard", "viewModelId", 0);
        float(b, "Artboard", "width", 240.0);
        float(b, "Artboard", "height", 100.0);
    });
    object(&mut b, "Shape", |b| uint(b, "Component", "parentId", 0));
    object(&mut b, "Rectangle", |b| {
        uint(b, "Component", "parentId", 1);
        float(b, "ParametricPath", "width", 200.0);
        float(b, "ParametricPath", "height", 60.0);
    });
    object(&mut b, "StateMachine", |_| {});
    object(&mut b, "StateMachineListenerSingle", |b| {
        uint(b, "StateMachineListener", "targetId", 1);
        uint(b, "StateMachineListenerSingle", "listenerTypeValue", 2);
    });
    for value in [300.0, 400.0] {
        object(&mut b, "BindablePropertyNumber", |b| {
            float(b, "BindablePropertyNumber", "propertyValue", value)
        });
        binding(
            &mut b,
            "BindablePropertyNumber",
            "propertyValue",
            0,
            None,
            1,
        );
        object(&mut b, "ListenerViewModelChange", |_| {});
    }
    object(&mut b, "BindablePropertyBoolean", |b| {
        uint(b, "BindablePropertyBoolean", "propertyValue", 1)
    });
    binding(
        &mut b,
        "BindablePropertyBoolean",
        "propertyValue",
        7,
        None,
        1,
    );
    object(&mut b, "ListenerViewModelChange", |_| {});
    b
}
