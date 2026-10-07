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

pub fn fixture() -> Vec<u8> {
    build(false)
}

pub fn group_fixture() -> Vec<u8> {
    build(true)
}

fn build(groups: bool) -> Vec<u8> {
    let mut b = b"RIVE".to_vec();
    for v in [7, 0, 3593, 0] {
        push_var_uint(&mut b, v);
    }
    object(&mut b, "Backboard", |_| {});
    object(&mut b, "ViewModel", |b| {
        string(b, "ViewModel", "name", "Values")
    });
    for (kind, name) in [
        ("Number", "n"),
        ("Boolean", "n_set"),
        ("Boolean", "b"),
        ("Boolean", "b_set"),
        ("Color", "c"),
        ("Boolean", "c_set"),
        ("String", "text"),
    ] {
        let kind = format!("ViewModelProperty{kind}");
        object(&mut b, &kind, |b| string(b, &kind, "name", name));
    }
    object(&mut b, "DataEnumCustom", |b| {
        string(b, "DataEnumCustom", "name", "Options")
    });
    for key in ["first", "second"] {
        object(&mut b, "DataEnumValue", |b| {
            string(b, "DataEnumValue", "key", key);
            string(b, "DataEnumValue", "value", key);
        });
    }
    object(&mut b, "ViewModelPropertyEnumCustom", |b| {
        string(b, "ViewModelPropertyEnumCustom", "name", "choice");
        uint(b, "ViewModelPropertyEnumCustom", "enumId", 0);
    });
    object(&mut b, "ViewModelPropertyBoolean", |b| {
        string(b, "ViewModelPropertyBoolean", "name", "choice_set")
    });
    if groups {
        for (kind, name) in [
            ("Boolean", "valid"),
            ("List", "n_errors"),
            ("List", "text_errors"),
            ("Boolean", "valid_text"),
        ] {
            let kind = format!("ViewModelProperty{kind}");
            object(&mut b, &kind, |b| string(b, &kind, "name", name));
        }
    }
    object(&mut b, "ViewModelInstance", |b| {
        uint(b, "ViewModelInstance", "viewModelId", 0)
    });
    for (i, kind) in [
        "Number", "Boolean", "Boolean", "Boolean", "Color", "Boolean", "String", "Enum", "Boolean",
    ]
    .iter()
    .enumerate()
    {
        let kind = format!("ViewModelInstance{kind}");
        object(&mut b, &kind, |b| {
            uint(b, &kind, "viewModelPropertyId", i as u64)
        });
    }
    if groups {
        for (index, kind) in [(9, "Boolean"), (10, "List"), (11, "List"), (12, "Boolean")] {
            let kind = format!("ViewModelInstance{kind}");
            object(&mut b, &kind, |b| {
                uint(b, &kind, "viewModelPropertyId", index)
            });
        }
    }
    object(&mut b, "ViewModel", |b| {
        string(b, "ViewModel", "name", "Container")
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
        uint(b, "ViewModelInstanceListItem", "viewModelId", 0);
        uint(b, "ViewModelInstanceListItem", "viewModelInstanceId", 0);
    });
    if groups {
        object(&mut b, "ViewModel", |b| {
            string(b, "ViewModel", "name", "ErrorEntry")
        });
        for name in ["code", "message"] {
            object(&mut b, "ViewModelPropertyString", |b| {
                string(b, "ViewModelPropertyString", "name", name)
            });
        }
        object(&mut b, "ViewModelInstance", |b| {
            uint(b, "ViewModelInstance", "viewModelId", 2)
        });
        for index in [0, 1] {
            object(&mut b, "ViewModelInstanceString", |b| {
                uint(b, "ViewModelInstanceString", "viewModelPropertyId", index)
            });
        }
    }
    object(&mut b, "Artboard", |b| {
        uint(b, "Artboard", "viewModelId", 0)
    });
    if groups {
        object(&mut b, "Artboard", |b| {
            uint(b, "Artboard", "viewModelId", 2)
        });
    }
    b
}
