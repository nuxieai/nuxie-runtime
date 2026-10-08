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

fn values() -> Vec<u8> {
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
    b
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
pub fn fixture(script: Option<&[u8]>, actions: &[Action], input: bool) -> Vec<u8> {
    let mut b = values();
    object(&mut b, "ViewModel", |b| {
        string(b, "ViewModel", "name", "Aux");
        uint(b, "ViewModel", "viewModelType", 2);
    });
    object(&mut b, "ViewModelInstance", |b| {
        uint(b, "ViewModelInstance", "viewModelId", 1);
    });
    object(&mut b, "DataConverterToString", |_| {});
    object(&mut b, "DataConverterToNumber", |_| {});
    if let Some(script) = script {
        object(&mut b, "ScriptAsset", |b| {
            uint(b, "ScriptAsset", "assetId", 0);
            string(b, "ScriptAsset", "name", "MarkerWrites");
        });
        object(&mut b, "FileAssetContents", |b| {
            blob(b, "FileAssetContents", "bytes", script)
        });
    }
    if input {
        object(&mut b, "FontAsset", |b| uint(b, "FontAsset", "assetId", 7));
        object(&mut b, "FileAssetContents", |b| {
            blob(
                b,
                "FileAssetContents",
                "bytes",
                include_bytes!("../../../../fixtures/fonts/roboto-input.ttf"),
            )
        });
    }
    object(&mut b, "Artboard", |b| {
        uint(b, "Artboard", "viewModelId", 0);
        float(b, "Artboard", "width", 240.0);
        float(b, "Artboard", "height", 100.0);
    });
    object(&mut b, "Shape", |b| uint(b, "Component", "parentId", 0)); // 1
    object(&mut b, "Rectangle", |b| {
        uint(b, "Component", "parentId", 1);
        float(b, "ParametricPath", "width", 200.0);
        float(b, "ParametricPath", "height", 60.0);
    }); // 2
    object(&mut b, "SemanticData", |b| {
        uint(b, "Component", "parentId", 1);
        uint(b, "SemanticData", "role", if input { 6 } else { 7 });
        string(b, "SemanticData", "label", "marker");
    }); // 3
    binding(&mut b, "SemanticData", "label", 1, Some(0), 0);
    if input {
        object(&mut b, "TextInput", |b| {
            uint(b, "Component", "parentId", 1);
            string(b, "Component", "name", "editable");
            string(b, "TextInput", "text", "0");
        }); // 4
        binding(&mut b, "TextInput", "text", 0, Some(0), 0);
        binding(&mut b, "TextInput", "text", 0, Some(1), 1);
        object(&mut b, "FocusData", |b| {
            uint(b, "Component", "parentId", 4);
            uint(b, "FocusData", "focusFlags", 7);
        });
        object(&mut b, "TextStylePaint", |b| {
            uint(b, "Component", "parentId", 4);
            uint(b, "TextStyle", "fontAssetId", u64::from(script.is_some()));
            float(b, "TextStyle", "fontSize", 24.0);
        });
    }
    object(&mut b, "StateMachine", |b| {
        string(b, "StateMachine", "name", "Main")
    });
    object(&mut b, "StateMachineListenerSingle", |b| {
        uint(b, "StateMachineListener", "targetId", 1);
        uint(b, "StateMachineListenerSingle", "listenerTypeValue", 2);
    });
    for action in actions {
        match action {
            Action::Number(value) => {
                object(&mut b, "BindablePropertyNumber", |b| {
                    float(b, "BindablePropertyNumber", "propertyValue", *value)
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
            Action::Marker(value) => {
                object(&mut b, "BindablePropertyBoolean", |b| {
                    uint(
                        b,
                        "BindablePropertyBoolean",
                        "propertyValue",
                        u64::from(*value),
                    )
                });
                binding(
                    &mut b,
                    "BindablePropertyBoolean",
                    "propertyValue",
                    1,
                    None,
                    1,
                );
                object(&mut b, "ListenerViewModelChange", |_| {});
            }
            Action::Script => object(&mut b, "ScriptedListenerAction", |b| {
                uint(b, "ScriptedListenerAction", "scriptAssetId", 0)
            }),
        }
    }
    if input {
        object(&mut b, "StateMachineListenerSingle", |b| {
            uint(b, "StateMachineListener", "targetId", 4);
            uint(b, "StateMachineListenerSingle", "listenerTypeValue", 8);
        });
    }
    b
}

#[cfg(feature = "scripting")]
pub fn occurrences(script: &[u8]) -> Vec<u8> {
    let mut b = values();
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
        uint(b, "ViewModelInstanceListItem", "viewModelId", 0);
        uint(b, "ViewModelInstanceListItem", "viewModelInstanceId", 0);
    });
    object(&mut b, "ScriptAsset", |b| {
        uint(b, "ScriptAsset", "assetId", 0);
        string(b, "ScriptAsset", "name", "OccurrenceWrites");
    });
    object(&mut b, "FileAssetContents", |b| {
        blob(b, "FileAssetContents", "bytes", script)
    });
    object(&mut b, "DataConverterToString", |_| {});
    object(&mut b, "Artboard", |b| {
        uint(b, "Artboard", "viewModelId", 0);
        float(b, "Artboard", "width", 60.0);
        float(b, "Artboard", "height", 30.0);
    });
    object(&mut b, "Shape", |b| uint(b, "Component", "parentId", 0));
    object(&mut b, "Rectangle", |b| {
        uint(b, "Component", "parentId", 1);
        float(b, "ParametricPath", "width", 50.0);
        float(b, "ParametricPath", "height", 20.0);
    });
    object(&mut b, "SemanticData", |b| {
        uint(b, "Component", "parentId", 1);
        uint(b, "SemanticData", "role", 7);
    });
    binding(&mut b, "SemanticData", "label", 1, Some(0), 0);
    object(&mut b, "Fill", |b| uint(b, "Component", "parentId", 1));
    object(&mut b, "SolidColor", |b| {
        uint(b, "Component", "parentId", 4);
        push_var_uint(b, u64::from(property_key("SolidColor", "colorValue")));
        b.extend_from_slice(&0xff336699_u32.to_le_bytes());
    });
    object(&mut b, "StateMachine", |b| {
        string(b, "StateMachine", "name", "Child")
    });
    object(&mut b, "StateMachineListenerSingle", |b| {
        uint(b, "StateMachineListener", "targetId", 1);
        uint(b, "StateMachineListenerSingle", "listenerTypeValue", 2);
    });
    object(&mut b, "ScriptedListenerAction", |b| {
        uint(b, "ScriptedListenerAction", "scriptAssetId", 0);
    });
    object(&mut b, "Artboard", |b| {
        uint(b, "Artboard", "viewModelId", 1);
        float(b, "Artboard", "width", 200.0);
        float(b, "Artboard", "height", 60.0);
    });
    object(&mut b, "NestedArtboard", |b| {
        uint(b, "Component", "parentId", 0);
        uint(b, "NestedArtboard", "artboardId", 0);
        float(b, "Node", "x", 100.0);
    }); // 1
    object(&mut b, "ViewModelInstance", |b| {
        uint(b, "Component", "parentId", 1);
        uint(b, "ViewModelInstance", "viewModelId", 0);
    }); // 2
    for (i, kind) in [
        "Number", "Boolean", "Boolean", "Boolean", "Color", "Boolean", "String", "Enum", "Boolean",
    ]
    .iter()
    .enumerate()
    {
        let kind = format!("ViewModelInstance{kind}");
        object(&mut b, &kind, |b| {
            uint(b, "Component", "parentId", 2);
            uint(b, &kind, "viewModelPropertyId", i as u64);
        });
    } // 3..11
    object(&mut b, "NestedStateMachine", |b| {
        uint(b, "Component", "parentId", 1);
        uint(b, "NestedStateMachine", "animationId", 0);
    }); // 12
    object(&mut b, "ArtboardComponentList", |b| {
        uint(b, "Component", "parentId", 0);
    }); // 13
    object(&mut b, "DataBindContext", |b| {
        uint(
            b,
            "DataBind",
            "propertyKey",
            u64::from(property_key("ArtboardComponentList", "listSource")),
        );
        blob(b, "DataBindContext", "sourcePathIds", &[1, 0]);
    });
    object(&mut b, "StateMachine", |b| {
        string(b, "StateMachine", "name", "Root")
    });
    b
}
