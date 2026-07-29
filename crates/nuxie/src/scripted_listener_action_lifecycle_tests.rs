use super::*;

use luaur_compiler::functions::luau_compile::luau_compile;
use nuxie_schema::definition_by_name;

fn compile_luau(source: &[u8]) -> Vec<u8> {
    luaur_common::set_all_flags(true);
    let mut output_size = 0;
    let output = luau_compile(
        source.as_ptr().cast(),
        source.len(),
        std::ptr::null_mut(),
        &mut output_size,
    );
    assert!(!output.is_null(), "pinned Luau compiler returned null");
    // SAFETY: the compiler returned a non-null allocation of output_size
    // bytes. Copying detaches the fixture from that allocation.
    unsafe { std::slice::from_raw_parts(output.cast(), output_size) }.to_vec()
}

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
    let definition = definition_by_name(type_name).expect("fixture type exists");
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            definition_by_name(ancestor)
                .expect("fixture ancestor exists")
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .expect("fixture property exists")
        .key
        .int
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(
            definition_by_name(type_name)
                .expect("fixture type exists")
                .type_key
                .int,
        ),
    );
    properties(bytes);
    push_var_uint(bytes, 0);
}

fn push_uint(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u64) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value);
}

fn push_f32(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: f32) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_color(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u32) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_blob(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &[u8]) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value);
}

fn push_string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
    push_blob(bytes, type_name, name, value.as_bytes());
}

fn push_amount_manifest(bytes: &mut Vec<u8>) {
    let mut names = Vec::new();
    push_var_uint(&mut names, 1);
    push_var_uint(&mut names, 7);
    push_var_uint(&mut names, 6);
    names.extend_from_slice(b"amount");

    let mut paths = Vec::new();
    push_var_uint(&mut paths, 1);
    push_var_uint(&mut paths, 3);
    push_var_uint(&mut paths, 1);
    push_var_uint(&mut paths, 7);

    let mut manifest = Vec::new();
    push_var_uint(&mut manifest, 0);
    push_var_uint(&mut manifest, names.len() as u64);
    manifest.extend_from_slice(&names);
    push_var_uint(&mut manifest, 1);
    push_var_uint(&mut manifest, paths.len() as u64);
    manifest.extend_from_slice(&paths);

    push_object(bytes, "ManifestAsset", |_| {});
    push_object(bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &manifest);
    });
}

fn scripted_listener_file(protocol_source: &[u8], action_count: usize) -> Vec<u8> {
    scripted_listener_file_with_folder_and_inputs(protocol_source, action_count, None, |_, _| {})
}

fn scripted_listener_file_with_inputs(
    protocol_source: &[u8],
    action_count: usize,
    mut push_inputs: impl FnMut(usize, &mut Vec<u8>),
) -> Vec<u8> {
    scripted_listener_file_with_folder_and_inputs(
        protocol_source,
        action_count,
        None,
        &mut push_inputs,
    )
}

fn scripted_listener_file_with_folder_and_inputs(
    protocol_source: &[u8],
    action_count: usize,
    folder: Option<&str>,
    mut push_inputs: impl FnMut(usize, &mut Vec<u8>),
) -> Vec<u8> {
    let mut protocol_payload = vec![0];
    protocol_payload.extend(compile_luau(protocol_source));

    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 9_401);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "ListenerLifecycle");
        if let Some(folder) = folder {
            push_string(bytes, "ScriptAsset", "folderPath", folder);
        }
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &protocol_payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
        push_f32(bytes, "Artboard", "height", 100.0);
    });
    push_object(&mut bytes, "Shape", |bytes| {
        push_uint(bytes, "Node", "parentId", 0);
    });
    push_object(&mut bytes, "Rectangle", |bytes| {
        push_uint(bytes, "Node", "parentId", 1);
        push_f32(bytes, "ParametricPath", "width", 100.0);
        push_f32(bytes, "ParametricPath", "height", 100.0);
    });
    push_object(&mut bytes, "StateMachine", |bytes| {
        push_string(bytes, "StateMachine", "name", "ListenerMachine");
    });
    push_object(&mut bytes, "StateMachineListenerSingle", |bytes| {
        push_uint(bytes, "StateMachineListener", "targetId", 1);
        push_uint(bytes, "StateMachineListenerSingle", "listenerTypeValue", 2);
    });
    for action_index in 0..action_count {
        push_object(&mut bytes, "ScriptedListenerAction", |bytes| {
            push_uint(bytes, "ScriptedListenerAction", "scriptAssetId", 0);
        });
        push_inputs(action_index, &mut bytes);
    }
    bytes
}

fn bound_listener_input_file(protocol_source: &[u8]) -> Vec<u8> {
    bound_listener_input_file_with_amount_flags(protocol_source, 0)
}

fn bound_listener_input_file_with_amount_flags(
    protocol_source: &[u8],
    amount_flags: u64,
) -> Vec<u8> {
    bound_listener_input_file_with_amount_options(protocol_source, amount_flags, false)
}

fn bound_listener_input_file_with_name_based_amount(protocol_source: &[u8]) -> Vec<u8> {
    const DATA_BIND_NAME_BASED: u64 = 1 << 4;
    bound_listener_input_file_with_amount_options(protocol_source, DATA_BIND_NAME_BASED, true)
}

fn bound_listener_input_file_with_amount_options(
    protocol_source: &[u8],
    amount_flags: u64,
    name_based_amount: bool,
) -> Vec<u8> {
    let mut protocol_payload = vec![0];
    protocol_payload.extend(compile_luau(protocol_source));
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 9_402);
    push_var_uint(&mut bytes, 0);

    push_object(&mut bytes, "ViewModel", |bytes| {
        push_string(bytes, "ViewModel", "name", "Root");
    });
    push_object(&mut bytes, "ViewModelPropertyNumber", |bytes| {
        push_string(bytes, "ViewModelPropertyNumber", "name", "amount");
    });
    push_object(&mut bytes, "ViewModelPropertyViewModel", |bytes| {
        push_string(bytes, "ViewModelPropertyViewModel", "name", "child");
        push_uint(
            bytes,
            "ViewModelPropertyViewModel",
            "viewModelReferenceId",
            1,
        );
    });
    push_object(&mut bytes, "ViewModelPropertyTrigger", |bytes| {
        push_string(bytes, "ViewModelPropertyTrigger", "name", "pulse");
    });
    push_object(&mut bytes, "ViewModel", |bytes| {
        push_string(bytes, "ViewModel", "name", "Child");
    });
    push_object(&mut bytes, "ViewModelPropertyNumber", |bytes| {
        push_string(bytes, "ViewModelPropertyNumber", "name", "score");
    });
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ViewModelInstance", |bytes| {
        push_string(bytes, "ViewModelInstance", "name", "child-default");
        push_uint(bytes, "ViewModelInstance", "viewModelId", 1);
    });
    push_object(&mut bytes, "ViewModelInstanceNumber", |bytes| {
        push_uint(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
        push_f32(bytes, "ViewModelInstanceNumber", "propertyValue", 4.0);
    });
    push_object(&mut bytes, "ViewModelInstance", |bytes| {
        push_string(bytes, "ViewModelInstance", "name", "root-default");
        push_uint(bytes, "ViewModelInstance", "viewModelId", 0);
    });
    push_object(&mut bytes, "ViewModelInstanceNumber", |bytes| {
        push_uint(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
        push_f32(bytes, "ViewModelInstanceNumber", "propertyValue", 9.5);
    });
    push_object(&mut bytes, "ViewModelInstanceViewModel", |bytes| {
        push_uint(
            bytes,
            "ViewModelInstanceViewModel",
            "viewModelPropertyId",
            1,
        );
        push_uint(bytes, "ViewModelInstanceViewModel", "propertyValue", 0);
    });
    push_object(&mut bytes, "ViewModelInstanceTrigger", |bytes| {
        push_uint(bytes, "ViewModelInstanceTrigger", "viewModelPropertyId", 2);
        push_uint(bytes, "ViewModelInstanceTrigger", "propertyValue", 0);
    });
    if name_based_amount {
        // C++ File retains ManifestAsset's DataResolver even after the
        // following ScriptAsset consumes its own FileAssetContents.
        push_amount_manifest(&mut bytes);
    }
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "BoundListener");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &protocol_payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
        push_f32(bytes, "Artboard", "height", 100.0);
        push_uint(bytes, "Artboard", "viewModelId", 0);
    });
    push_object(&mut bytes, "Shape", |bytes| {
        push_uint(bytes, "Node", "parentId", 0);
    });
    push_object(&mut bytes, "Rectangle", |bytes| {
        push_uint(bytes, "Node", "parentId", 1);
        push_f32(bytes, "ParametricPath", "width", 100.0);
        push_f32(bytes, "ParametricPath", "height", 100.0);
    });
    push_object(&mut bytes, "StateMachine", |bytes| {
        push_string(bytes, "StateMachine", "name", "ListenerMachine");
    });
    push_object(&mut bytes, "StateMachineListenerSingle", |bytes| {
        push_uint(bytes, "StateMachineListener", "targetId", 1);
        push_uint(bytes, "StateMachineListenerSingle", "listenerTypeValue", 2);
    });
    push_object(&mut bytes, "ScriptedListenerAction", |bytes| {
        push_uint(bytes, "ScriptedListenerAction", "scriptAssetId", 0);
    });

    let mut amount_path = Vec::new();
    if name_based_amount {
        push_var_uint(&mut amount_path, 3);
    } else {
        push_var_uint(&mut amount_path, 0);
        push_var_uint(&mut amount_path, 0);
    }
    push_object(&mut bytes, "ScriptInputNumber", |bytes| {
        push_string(bytes, "ScriptInputNumber", "name", "boundAmount");
        push_f32(bytes, "ScriptInputNumber", "propertyValue", 1.0);
    });
    push_object(&mut bytes, "DataBindContext", |bytes| {
        push_uint(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key("ScriptInputNumber", "propertyValue")),
        );
        push_blob(bytes, "DataBindContext", "sourcePathIds", &amount_path);
        if amount_flags != 0 {
            push_uint(bytes, "DataBindContext", "flags", amount_flags);
        }
    });

    let mut child_path = Vec::new();
    push_var_uint(&mut child_path, 0);
    push_var_uint(&mut child_path, 1);
    push_object(&mut bytes, "ScriptInputViewModelProperty", |bytes| {
        push_string(bytes, "ScriptInputViewModelProperty", "name", "boundChild");
        push_blob(
            bytes,
            "ScriptInputViewModelProperty",
            "dataBindPathIds",
            &child_path,
        );
    });

    let mut trigger_path = Vec::new();
    push_var_uint(&mut trigger_path, 0);
    push_var_uint(&mut trigger_path, 2);
    push_object(&mut bytes, "ScriptInputTrigger", |bytes| {
        push_string(bytes, "ScriptInputTrigger", "name", "boundPulse");
    });
    push_object(&mut bytes, "DataBindContext", |bytes| {
        push_uint(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key("ScriptInputTrigger", "propertyValue")),
        );
        push_blob(bytes, "DataBindContext", "sourcePathIds", &trigger_path);
    });
    bytes
}

fn bound_listener_scripted_converter_file(
    listener_source: &[u8],
    converter_source: &[u8],
) -> Vec<u8> {
    bound_listener_scripted_converter_file_with_options(
        listener_source,
        converter_source,
        false,
        None,
    )
}

fn bound_listener_scripted_converter_file_with_missing_child(
    listener_source: &[u8],
    converter_source: &[u8],
) -> Vec<u8> {
    bound_listener_scripted_converter_file_with_options(
        listener_source,
        converter_source,
        true,
        None,
    )
}

fn bound_listener_scripted_converter_file_with_options(
    listener_source: &[u8],
    converter_source: &[u8],
    include_missing_child: bool,
    converter_implemented_methods: Option<u64>,
) -> Vec<u8> {
    bound_listener_scripted_converter_file_with_manifest_options(
        listener_source,
        converter_source,
        include_missing_child,
        converter_implemented_methods,
        false,
        false,
    )
}

fn bound_listener_scripted_converter_file_with_name_based_custom_amount(
    listener_source: &[u8],
    converter_source: &[u8],
) -> Vec<u8> {
    bound_listener_scripted_converter_file_with_manifest_options(
        listener_source,
        converter_source,
        false,
        None,
        true,
        false,
    )
}

fn bound_listener_scripted_converter_file_with_listener_child(
    listener_source: &[u8],
    converter_source: &[u8],
) -> Vec<u8> {
    bound_listener_scripted_converter_file_with_manifest_options(
        listener_source,
        converter_source,
        false,
        None,
        false,
        true,
    )
}

fn bound_listener_scripted_converter_file_with_manifest_options(
    listener_source: &[u8],
    converter_source: &[u8],
    include_missing_child: bool,
    converter_implemented_methods: Option<u64>,
    name_based_custom_amount: bool,
    include_listener_child: bool,
) -> Vec<u8> {
    let mut listener_payload = vec![0];
    listener_payload.extend(compile_luau(listener_source));
    let mut converter_payload = vec![0];
    converter_payload.extend(compile_luau(converter_source));
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 9_405);
    push_var_uint(&mut bytes, 0);

    push_object(&mut bytes, "ViewModel", |bytes| {
        push_string(bytes, "ViewModel", "name", "Root");
    });
    push_object(&mut bytes, "ViewModelPropertyNumber", |bytes| {
        push_string(bytes, "ViewModelPropertyNumber", "name", "amount");
    });
    if include_missing_child || include_listener_child {
        push_object(&mut bytes, "ViewModelPropertyViewModel", |bytes| {
            push_string(bytes, "ViewModelPropertyViewModel", "name", "child");
            push_uint(
                bytes,
                "ViewModelPropertyViewModel",
                "viewModelReferenceId",
                1,
            );
        });
        push_object(&mut bytes, "ViewModel", |bytes| {
            push_string(bytes, "ViewModel", "name", "Child");
        });
        push_object(&mut bytes, "ViewModelPropertyNumber", |bytes| {
            push_string(bytes, "ViewModelPropertyNumber", "name", "score");
        });
    }
    push_object(&mut bytes, "Backboard", |_| {});
    if include_listener_child {
        push_object(&mut bytes, "ViewModelInstance", |bytes| {
            push_string(bytes, "ViewModelInstance", "name", "child-default");
            push_uint(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object(&mut bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_f32(bytes, "ViewModelInstanceNumber", "propertyValue", 4.0);
        });
    }
    push_object(&mut bytes, "ViewModelInstance", |bytes| {
        push_string(bytes, "ViewModelInstance", "name", "root-default");
        push_uint(bytes, "ViewModelInstance", "viewModelId", 0);
    });
    push_object(&mut bytes, "ViewModelInstanceNumber", |bytes| {
        push_uint(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
        push_f32(bytes, "ViewModelInstanceNumber", "propertyValue", 9.5);
    });
    if include_missing_child {
        push_object(&mut bytes, "ViewModelInstanceViewModel", |bytes| {
            push_uint(
                bytes,
                "ViewModelInstanceViewModel",
                "viewModelPropertyId",
                1,
            );
            push_uint(bytes, "ViewModelInstanceViewModel", "propertyValue", 999);
        });
    } else if include_listener_child {
        push_object(&mut bytes, "ViewModelInstanceViewModel", |bytes| {
            push_uint(
                bytes,
                "ViewModelInstanceViewModel",
                "viewModelPropertyId",
                1,
            );
            push_uint(bytes, "ViewModelInstanceViewModel", "propertyValue", 0);
        });
    }
    if name_based_custom_amount {
        // Keep the resolver before both scripts so this fixture detects any
        // importer that replaces ManifestAsset's persistent File resolver
        // with later ScriptAsset contents.
        push_amount_manifest(&mut bytes);
    }
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "ConvertedListener");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &listener_payload);
    });
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 1);
        push_string(bytes, "ScriptAsset", "name", "ListenerConverter");
        if let Some(methods) = converter_implemented_methods {
            push_uint(
                bytes,
                "ScriptAsset",
                "serializedImplementedMethods",
                methods,
            );
        }
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &converter_payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
        push_f32(bytes, "Artboard", "height", 100.0);
        push_uint(bytes, "Artboard", "viewModelId", 0);
    });
    push_object(&mut bytes, "Shape", |bytes| {
        push_uint(bytes, "Node", "parentId", 0);
    });
    push_object(&mut bytes, "Rectangle", |bytes| {
        push_uint(bytes, "Node", "parentId", 1);
        push_f32(bytes, "ParametricPath", "width", 100.0);
        push_f32(bytes, "ParametricPath", "height", 100.0);
    });
    push_object(&mut bytes, "StateMachine", |bytes| {
        push_string(bytes, "StateMachine", "name", "ListenerMachine");
    });
    push_object(&mut bytes, "StateMachineListenerSingle", |bytes| {
        push_uint(bytes, "StateMachineListener", "targetId", 1);
        push_uint(bytes, "StateMachineListenerSingle", "listenerTypeValue", 2);
    });
    push_object(&mut bytes, "ScriptedListenerAction", |bytes| {
        push_uint(bytes, "ScriptedListenerAction", "scriptAssetId", 0);
    });

    let mut source_path = Vec::new();
    push_var_uint(&mut source_path, 0);
    push_var_uint(&mut source_path, 0);
    push_object(&mut bytes, "ScriptInputNumber", |bytes| {
        push_string(bytes, "ScriptInputNumber", "name", "convertedAmount");
        push_f32(bytes, "ScriptInputNumber", "propertyValue", 2.0);
    });
    push_object(&mut bytes, "DataBindContext", |bytes| {
        push_uint(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key("ScriptInputNumber", "propertyValue")),
        );
        push_blob(bytes, "DataBindContext", "sourcePathIds", &source_path);
        push_uint(bytes, "DataBindContext", "converterId", 0);
    });
    if include_listener_child {
        let mut child_path = Vec::new();
        push_var_uint(&mut child_path, 0);
        push_var_uint(&mut child_path, 1);
        push_object(&mut bytes, "ScriptInputViewModelProperty", |bytes| {
            push_string(bytes, "ScriptInputViewModelProperty", "name", "boundChild");
            push_blob(
                bytes,
                "ScriptInputViewModelProperty",
                "dataBindPathIds",
                &child_path,
            );
        });
    }

    // This ScriptedObject boundary ends the listener's input collection. Its
    // following ScriptInput/DataBind records belong to this concrete converter
    // occurrence, exactly as ScriptedObjectImporter assigns them in C++.
    push_object(&mut bytes, "ScriptedDataConverter", |bytes| {
        push_uint(bytes, "ScriptedDataConverter", "scriptAssetId", 1);
    });
    push_object(&mut bytes, "ScriptInputNumber", |bytes| {
        push_string(bytes, "ScriptInputNumber", "name", "customAmount");
        push_f32(bytes, "ScriptInputNumber", "propertyValue", 1.0);
    });
    let mut custom_source_path = Vec::new();
    if name_based_custom_amount {
        push_var_uint(&mut custom_source_path, 3);
    } else {
        custom_source_path.extend_from_slice(&source_path);
    }
    push_object(&mut bytes, "DataBindContext", |bytes| {
        push_uint(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key("ScriptInputNumber", "propertyValue")),
        );
        push_blob(
            bytes,
            "DataBindContext",
            "sourcePathIds",
            &custom_source_path,
        );
        if name_based_custom_amount {
            push_uint(bytes, "DataBindContext", "flags", 1 << 4);
        }
    });
    if include_missing_child {
        let mut child_path = Vec::new();
        push_var_uint(&mut child_path, 0);
        push_var_uint(&mut child_path, 1);
        push_object(&mut bytes, "ScriptInputViewModelProperty", |bytes| {
            push_string(bytes, "ScriptInputViewModelProperty", "name", "customChild");
            push_blob(
                bytes,
                "ScriptInputViewModelProperty",
                "dataBindPathIds",
                &child_path,
            );
        });
    }
    bytes
}

fn ordinary_state_machine_scripted_converter_file(converter_source: &[u8]) -> Vec<u8> {
    let mut converter_payload = vec![0];
    converter_payload.extend(compile_luau(converter_source));
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 9_406);
    push_var_uint(&mut bytes, 0);

    push_object(&mut bytes, "ViewModel", |bytes| {
        push_string(bytes, "ViewModel", "name", "Root");
    });
    push_object(&mut bytes, "ViewModelPropertyNumber", |bytes| {
        push_string(bytes, "ViewModelPropertyNumber", "name", "amount");
    });
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ViewModelInstance", |bytes| {
        push_string(bytes, "ViewModelInstance", "name", "root-default");
        push_uint(bytes, "ViewModelInstance", "viewModelId", 0);
    });
    push_object(&mut bytes, "ViewModelInstanceNumber", |bytes| {
        push_uint(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
        push_f32(bytes, "ViewModelInstanceNumber", "propertyValue", 9.5);
    });
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "OrdinaryConverter");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &converter_payload);
    });
    push_object(&mut bytes, "ScriptedDataConverter", |bytes| {
        push_uint(bytes, "ScriptedDataConverter", "scriptAssetId", 0);
    });

    let mut source_path = Vec::new();
    push_var_uint(&mut source_path, 0);
    push_var_uint(&mut source_path, 0);
    push_object(&mut bytes, "ScriptInputNumber", |bytes| {
        push_string(bytes, "ScriptInputNumber", "name", "customAmount");
        push_f32(bytes, "ScriptInputNumber", "propertyValue", 1.0);
    });
    push_object(&mut bytes, "DataBindContext", |bytes| {
        push_uint(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key("ScriptInputNumber", "propertyValue")),
        );
        push_blob(bytes, "DataBindContext", "sourcePathIds", &source_path);
    });

    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
        push_f32(bytes, "Artboard", "height", 100.0);
        push_uint(bytes, "Artboard", "viewModelId", 0);
    });
    push_object(&mut bytes, "Shape", |bytes| {
        push_uint(bytes, "Node", "parentId", 0);
    });
    push_object(&mut bytes, "Rectangle", |bytes| {
        push_uint(bytes, "Node", "parentId", 1);
        push_f32(bytes, "ParametricPath", "width", 100.0);
        push_f32(bytes, "ParametricPath", "height", 100.0);
    });
    push_object(&mut bytes, "StateMachine", |bytes| {
        push_string(bytes, "StateMachine", "name", "OrdinaryConverterMachine");
    });
    for value in [2.0, 3.0] {
        push_object(&mut bytes, "BindablePropertyNumber", |bytes| {
            push_f32(bytes, "BindablePropertyNumber", "propertyValue", value);
        });
        push_object(&mut bytes, "DataBindContext", |bytes| {
            push_uint(
                bytes,
                "DataBindContext",
                "propertyKey",
                u64::from(property_key("BindablePropertyNumber", "propertyValue")),
            );
            push_blob(bytes, "DataBindContext", "sourcePathIds", &source_path);
            push_uint(bytes, "DataBindContext", "converterId", 0);
        });
    }
    bytes
}

fn ordinary_scripted_converter_replaces_nested_source_file(converter_source: &[u8]) -> Vec<u8> {
    let mut converter_payload = vec![0];
    converter_payload.extend(compile_luau(converter_source));
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 9_407);
    push_var_uint(&mut bytes, 0);

    push_object(&mut bytes, "ViewModel", |bytes| {
        push_string(bytes, "ViewModel", "name", "Root");
    });
    push_object(&mut bytes, "ViewModelPropertyViewModel", |bytes| {
        push_string(bytes, "ViewModelPropertyViewModel", "name", "child");
        push_uint(
            bytes,
            "ViewModelPropertyViewModel",
            "viewModelReferenceId",
            1,
        );
    });
    push_object(&mut bytes, "ViewModelPropertyNumber", |bytes| {
        push_string(bytes, "ViewModelPropertyNumber", "name", "kick");
    });
    push_object(&mut bytes, "ViewModel", |bytes| {
        push_string(bytes, "ViewModel", "name", "Child");
    });
    push_object(&mut bytes, "ViewModelPropertyNumber", |bytes| {
        push_string(bytes, "ViewModelPropertyNumber", "name", "leaf");
    });
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ViewModelInstance", |bytes| {
        push_string(bytes, "ViewModelInstance", "name", "child-old");
        push_uint(bytes, "ViewModelInstance", "viewModelId", 1);
    });
    push_object(&mut bytes, "ViewModelInstanceNumber", |bytes| {
        push_uint(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
        push_f32(bytes, "ViewModelInstanceNumber", "propertyValue", 11.0);
    });
    push_object(&mut bytes, "ViewModelInstance", |bytes| {
        push_string(bytes, "ViewModelInstance", "name", "child-new");
        push_uint(bytes, "ViewModelInstance", "viewModelId", 1);
    });
    push_object(&mut bytes, "ViewModelInstanceNumber", |bytes| {
        push_uint(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
        push_f32(bytes, "ViewModelInstanceNumber", "propertyValue", 22.0);
    });
    push_object(&mut bytes, "ViewModelInstance", |bytes| {
        push_string(bytes, "ViewModelInstance", "name", "root-default");
        push_uint(bytes, "ViewModelInstance", "viewModelId", 0);
    });
    push_object(&mut bytes, "ViewModelInstanceViewModel", |bytes| {
        push_uint(
            bytes,
            "ViewModelInstanceViewModel",
            "viewModelPropertyId",
            0,
        );
        push_uint(bytes, "ViewModelInstanceViewModel", "propertyValue", 0);
    });
    push_object(&mut bytes, "ViewModelInstanceNumber", |bytes| {
        push_uint(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 1);
        push_f32(bytes, "ViewModelInstanceNumber", "propertyValue", 5.0);
    });
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "ReplacementConverter");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &converter_payload);
    });
    push_object(&mut bytes, "ScriptedDataConverter", |bytes| {
        push_uint(bytes, "ScriptedDataConverter", "scriptAssetId", 0);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
        push_f32(bytes, "Artboard", "height", 100.0);
        push_uint(bytes, "Artboard", "viewModelId", 0);
    });
    push_object(&mut bytes, "Shape", |bytes| {
        push_uint(bytes, "Node", "parentId", 0);
    });
    push_object(&mut bytes, "Rectangle", |bytes| {
        push_uint(bytes, "Node", "parentId", 1);
        push_f32(bytes, "ParametricPath", "width", 100.0);
        push_f32(bytes, "ParametricPath", "height", 100.0);
    });
    push_object(&mut bytes, "StateMachine", |bytes| {
        push_string(bytes, "StateMachine", "name", "ReplacementConverterMachine");
    });

    let mut kick_path = Vec::new();
    push_var_uint(&mut kick_path, 0);
    push_var_uint(&mut kick_path, 1);
    push_object(&mut bytes, "BindablePropertyNumber", |bytes| {
        push_f32(bytes, "BindablePropertyNumber", "propertyValue", 0.0);
    });
    push_object(&mut bytes, "DataBindContext", |bytes| {
        push_uint(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key("BindablePropertyNumber", "propertyValue")),
        );
        push_blob(bytes, "DataBindContext", "sourcePathIds", &kick_path);
        push_uint(bytes, "DataBindContext", "converterId", 0);
    });

    let mut leaf_path = Vec::new();
    push_var_uint(&mut leaf_path, 0);
    push_var_uint(&mut leaf_path, 0);
    push_var_uint(&mut leaf_path, 0);
    push_object(&mut bytes, "BindablePropertyNumber", |bytes| {
        push_f32(bytes, "BindablePropertyNumber", "propertyValue", 0.0);
    });
    push_object(&mut bytes, "DataBindContext", |bytes| {
        push_uint(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key("BindablePropertyNumber", "propertyValue")),
        );
        push_blob(bytes, "DataBindContext", "sourcePathIds", &leaf_path);
    });
    bytes
}

fn pointer_trigger_binding_file(protocol_source: &[u8]) -> Vec<u8> {
    pointer_trigger_binding_file_for_listener(protocol_source, 2)
}

fn pointer_trigger_binding_file_for_listener(
    protocol_source: &[u8],
    listener_type_value: u64,
) -> Vec<u8> {
    const DATA_BIND_TO_SOURCE: u64 = 1 << 0;

    let mut protocol_payload = vec![0];
    protocol_payload.extend(compile_luau(protocol_source));
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 9_404);
    push_var_uint(&mut bytes, 0);

    push_object(&mut bytes, "ViewModel", |bytes| {
        push_string(bytes, "ViewModel", "name", "Root");
    });
    push_object(&mut bytes, "ViewModelPropertyTrigger", |bytes| {
        push_string(bytes, "ViewModelPropertyTrigger", "name", "pulse");
    });
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ViewModelInstance", |bytes| {
        push_string(bytes, "ViewModelInstance", "name", "root-default");
        push_uint(bytes, "ViewModelInstance", "viewModelId", 0);
    });
    push_object(&mut bytes, "ViewModelInstanceTrigger", |bytes| {
        push_uint(bytes, "ViewModelInstanceTrigger", "viewModelPropertyId", 0);
        push_uint(bytes, "ViewModelInstanceTrigger", "propertyValue", 0);
    });
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "PointerTriggerListener");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &protocol_payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
        push_f32(bytes, "Artboard", "height", 100.0);
        push_uint(bytes, "Artboard", "viewModelId", 0);
    });
    push_object(&mut bytes, "Shape", |bytes| {
        push_uint(bytes, "Node", "parentId", 0);
    });
    push_object(&mut bytes, "Rectangle", |bytes| {
        push_uint(bytes, "Node", "parentId", 1);
        push_f32(bytes, "ParametricPath", "width", 100.0);
        push_f32(bytes, "ParametricPath", "height", 100.0);
    });
    push_object(&mut bytes, "StateMachine", |bytes| {
        push_string(bytes, "StateMachine", "name", "ListenerMachine");
    });
    push_object(&mut bytes, "StateMachineListenerSingle", |bytes| {
        push_uint(bytes, "StateMachineListener", "targetId", 1);
        push_uint(
            bytes,
            "StateMachineListenerSingle",
            "listenerTypeValue",
            listener_type_value,
        );
    });

    let mut trigger_path = Vec::new();
    push_var_uint(&mut trigger_path, 0);
    push_var_uint(&mut trigger_path, 0);
    push_object(&mut bytes, "BindablePropertyTrigger", |bytes| {
        push_uint(bytes, "BindablePropertyTrigger", "propertyValue", 1);
    });
    push_object(&mut bytes, "DataBindContext", |bytes| {
        push_uint(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key("BindablePropertyTrigger", "propertyValue")),
        );
        push_blob(bytes, "DataBindContext", "sourcePathIds", &trigger_path);
        push_uint(bytes, "DataBindContext", "flags", DATA_BIND_TO_SOURCE);
    });
    push_object(&mut bytes, "ListenerViewModelChange", |_| {});
    push_object(&mut bytes, "ScriptedListenerAction", |bytes| {
        push_uint(bytes, "ScriptedListenerAction", "scriptAssetId", 0);
    });
    push_object(&mut bytes, "ScriptInputTrigger", |bytes| {
        push_string(bytes, "ScriptInputTrigger", "name", "boundPulse");
    });
    push_object(&mut bytes, "DataBindContext", |bytes| {
        push_uint(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key("ScriptInputTrigger", "propertyValue")),
        );
        push_blob(bytes, "DataBindContext", "sourcePathIds", &trigger_path);
    });
    bytes
}

fn module_retry_listener_file() -> Vec<u8> {
    fn payload(source: &[u8]) -> Vec<u8> {
        let mut payload = vec![0];
        payload.extend(compile_luau(source));
        payload
    }

    let module_a = payload(
        br#"
            local nuxie = require("nuxie")
            nuxie.trigger("module_a_registered")
            local _dependency = require("B")
            return {}
        "#,
    );
    let module_b = payload(br#"return {}"#);
    let protocol = payload(
        br#"
            local _module = require("A")
            return function(_context)
                return { performAction = function(_self, _invocation) end }
            end
        "#,
    );
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 9_403);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    for (asset_id, name, is_module, contents) in [
        (0, "A", true, module_a.as_slice()),
        (1, "B", true, module_b.as_slice()),
        (2, "RetryListener", false, protocol.as_slice()),
    ] {
        push_object(&mut bytes, "ScriptAsset", |bytes| {
            push_uint(bytes, "ScriptAsset", "assetId", asset_id);
            push_string(bytes, "ScriptAsset", "name", name);
            if is_module {
                push_uint(bytes, "ScriptAsset", "isModule", 1);
            }
        });
        push_object(&mut bytes, "FileAssetContents", |bytes| {
            push_blob(bytes, "FileAssetContents", "bytes", contents);
        });
    }
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
        push_f32(bytes, "Artboard", "height", 100.0);
    });
    push_object(&mut bytes, "Shape", |bytes| {
        push_uint(bytes, "Node", "parentId", 0);
    });
    push_object(&mut bytes, "Rectangle", |bytes| {
        push_uint(bytes, "Node", "parentId", 1);
        push_f32(bytes, "ParametricPath", "width", 100.0);
        push_f32(bytes, "ParametricPath", "height", 100.0);
    });
    push_object(&mut bytes, "StateMachine", |_| {});
    push_object(&mut bytes, "StateMachineListenerSingle", |bytes| {
        push_uint(bytes, "StateMachineListener", "targetId", 1);
        push_uint(bytes, "StateMachineListenerSingle", "listenerTypeValue", 2);
    });
    push_object(&mut bytes, "ScriptedListenerAction", |bytes| {
        push_uint(bytes, "ScriptedListenerAction", "scriptAssetId", 2);
    });
    bytes
}

fn prepared_machine(
    protocol_source: &[u8],
    action_count: usize,
) -> (
    Arc<File>,
    OwnedArtboardInstance,
    StateMachineInstance,
    RecordingFactory,
) {
    let bytes = scripted_listener_file(protocol_source, action_count);
    let runtime = read_runtime_file_for_facade(&bytes).expect("import scripted listener fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build scripted listener file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate scripted listener artboard");
    let machine = instance
        .default_state_machine_instance()
        .expect("instantiate scripted listener state machine");
    (file, instance, machine, RecordingFactory::new())
}

#[test]
fn public_machine_construction_synchronously_prepares_scripted_data_without_blocking_ordinary_input()
 {
    let bytes = scripted_listener_file(
        br#"
            return function(_context)
                return {
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        1,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import public construction fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build public construction file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(file)
        .expect("instantiate public construction artboard");
    let mut factory = RecordingFactory::new();
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("make the public runtime file resolver available");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("the public constructor synchronously prepares scripted data");

    assert!(
        machine.scripted_object_initialization_complete(),
        "a machine returned by the public facade must not expose its preparation window"
    );
    let rust_pointer_hit = machine.pointer_down(&mut instance.raw, 50.0, 50.0, 1);
    assert!(
        rust_pointer_hit,
        "an ordinary authored pointer listener remains live immediately after construction"
    );
    let rust_advanced = machine
        .advance_and_apply(&mut instance.raw, 0.25)
        .expect("ordinary advance remains callable immediately after construction");

    if let Some(probe) = scripted_cpp_probe_path() {
        let fixture = std::env::temp_dir().join(format!(
            "nuxie-scripted-mount-ordinary-input-{}.riv",
            std::process::id()
        ));
        std::fs::write(&fixture, &bytes).expect("write scripted mount differential fixture");
        let output = std::process::Command::new(&probe)
            .arg("--instance-artboards")
            .arg("--runtime-pointer-down-state-machine")
            .arg("0")
            .arg("50")
            .arg("50")
            .arg("--runtime-advance-and-apply-state-machine")
            .arg("0")
            .arg("0.25")
            .arg("--file")
            .arg(&fixture)
            .output();
        let _ = std::fs::remove_file(&fixture);
        let output = output.expect("run scripted mount C++ differential");
        assert!(
            output.status.success(),
            "scripted mount C++ differential failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("decode scripted mount C++ differential");
        let reports = report
            .get("artboards")
            .and_then(serde_json::Value::as_array)
            .and_then(|artboards| artboards.first())
            .and_then(|artboard| artboard.get("runtimeStateMachineAdvances"))
            .and_then(serde_json::Value::as_array)
            .expect("C++ scripted mount reports");
        assert_eq!(reports.len(), 2);
        let cpp_pointer_hit = reports[0]
            .get("pointerHitResult")
            .and_then(serde_json::Value::as_i64)
            .is_some_and(|result| result != 0);
        let cpp_advanced = reports[1]
            .get("advanced")
            .and_then(serde_json::Value::as_bool)
            .expect("C++ advanceAndApply result");
        assert_eq!(rust_pointer_hit, cpp_pointer_hit);
        assert_eq!(rust_advanced, cpp_advanced);
    }
}

fn scripted_cpp_probe_path() -> Option<std::path::PathBuf> {
    if let Some(path) = std::env::var_os("RIVE_CPP_PROBE_SCRIPTED") {
        let path = std::path::PathBuf::from(path);
        return Some(if path.is_absolute() {
            path
        } else {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(path)
        });
    }
    let os = match std::env::consts::OS {
        "macos" => "macosx",
        other => other,
    };
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tools/cpp-probe/build")
        .join(os)
        .join("bin/debug/rive_cpp_probe_scripted");
    path.exists().then_some(path)
}

#[test]
fn listener_init_uses_the_generator_context_once_before_first_perform() {
    let bytes = scripted_listener_file(
        br#"
            local nuxie = require("nuxie")

            return function(generatorContext)
                return {
                    initialized = false,
                    init = function(self, initContext)
                        if generatorContext ~= initContext then
                            error("listener init received a different Context")
                        end
                        if self.initialized then
                            error("listener init ran twice")
                        end
                        self.initialized = true
                        nuxie.trigger("listener_init")
                        return true
                    end,
                    performAction = function(self, _invocation)
                        if not self.initialized then
                            error("listener performed before init")
                        end
                        nuxie.trigger("listener_perform")
                    end,
                }
            end
        "#,
        1,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import scripted listener fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build scripted listener file"));
    let mut factory = RecordingFactory::new();
    let (mut session, creation) = flow_session::FlowSession::create_with_factory(
        file,
        flow_session::FlowSessionConfig::default(),
        &mut factory,
    )
    .expect("create the scripted listener flow session");

    let creation_commands = creation
        .outputs
        .iter()
        .filter(|output| {
            matches!(
                output.payload,
                flow_session::FlowOutputPayload::HostCommand { .. }
            )
        })
        .collect::<Vec<_>>();
    let creation_command = creation_commands
        .first()
        .expect("listener init emits one creation command");
    assert_eq!(creation_commands.len(), 1);
    assert_eq!(creation_command.cycle, 0);
    assert_eq!(
        creation_command.phase,
        flow_session::FlowOutputPhase::HostWork
    );
    assert!(matches!(
        &creation_command.payload,
        flow_session::FlowOutputPayload::HostCommand { name, .. }
            if name == "listener_init"
    ));

    let result = session
        .perform_with_factory(
            flow_session::FlowOperation::PointerBatch(flow_session::FlowPointerBatch {
                events: vec![
                    flow_session::FlowPointerEvent {
                        kind: flow_session::FlowPointerKind::Down,
                        pointer_id: 1,
                        x: 0.0,
                        y: 0.0,
                        timestamp_seconds: 0.0,
                    },
                    flow_session::FlowPointerEvent {
                        kind: flow_session::FlowPointerKind::Down,
                        pointer_id: 2,
                        x: 0.0,
                        y: 0.0,
                        timestamp_seconds: 0.0,
                    },
                ],
            }),
            &mut factory,
        )
        .expect("perform the scripted listener twice");
    let perform_commands = result
        .outputs
        .iter()
        .filter_map(|output| match &output.payload {
            flow_session::FlowOutputPayload::HostCommand { name, .. } => {
                Some((output.phase, name.as_str()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        perform_commands,
        [
            (flow_session::FlowOutputPhase::HostWork, "listener_perform"),
            (flow_session::FlowOutputPhase::HostWork, "listener_perform"),
        ]
    );
}

#[test]
fn unbound_listener_stops_after_cpp_constructor_two_attempts() {
    let bytes = scripted_listener_file(
        br#"
            generated = generated or 0

            return function(_context)
                generated += 1
                if generated <= 2 then
                    return nil
                end
                return {
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        1,
    );

    if let Some(probe) = scripted_cpp_probe_path() {
        let fixture = std::env::temp_dir().join(format!(
            "nuxie-scripted-listener-two-attempts-{}.riv",
            std::process::id()
        ));
        std::fs::write(&fixture, &bytes).expect("write scripted C++ differential fixture");
        let output = std::process::Command::new(&probe)
            .arg("--instance-artboards")
            .arg("--runtime-snapshot-state-machine-scripts")
            .arg("0")
            .arg("--file")
            .arg(&fixture)
            .output();
        let _ = std::fs::remove_file(&fixture);
        let output = output.expect("run scripted C++ differential");
        assert!(
            output.status.success(),
            "scripted C++ differential failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("decode scripted C++ differential");
        let reports = report
            .get("artboards")
            .and_then(serde_json::Value::as_array)
            .and_then(|artboards| artboards.first())
            .and_then(|artboard| artboard.get("runtimeStateMachineAdvances"))
            .and_then(serde_json::Value::as_array)
            .expect("C++ scripted-listener lifecycle reports");
        let cold_occurrence = reports
            .first()
            .and_then(|advance| advance.get("scriptedObjects"))
            .and_then(serde_json::Value::as_array)
            .and_then(|objects| objects.first())
            .expect("C++ unbound scripted-listener occurrence report");
        assert!(
            cold_occurrence
                .get("occurrenceTableOrdinal")
                .is_some_and(serde_json::Value::is_null),
            "pinned C++ must leave the occurrence inert after exactly two failed constructor attempts"
        );
        assert_eq!(
            cold_occurrence
                .get("userLuaInitDone")
                .and_then(serde_json::Value::as_bool),
            Some(false)
        );
    }

    let runtime =
        read_runtime_file_for_facade(&bytes).expect("import unbound scripted-listener fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build unbound scripted-listener file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate unbound scripted-listener artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate unbound scripted-listener machine");
    let definition = machine
        .scripted_objects()
        .first()
        .expect("scripted-listener definition")
        .scripted_object_global_id();
    let mut factory = RecordingFactory::new();
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("bootstrap unbound scripted-listener file VM");

    instantiate_script_listener_actions(&file, &mut machine, &mut factory, None)
        .expect("run unbound scripted-listener constructor");
    assert!(
        !machine.has_scripted_object_instance(definition),
        "Rust must not invent a third live-context attempt when no DataContext exists (`state_machine_instance.cpp:2072-2082`; `artboard.cpp:2844-2856`)"
    );
    assert!(
        machine.bind_owned_view_model_contexts(&nuxie_runtime::RuntimeOwnedViewModelContext::new()),
        "install the first genuine empty DataContext"
    );
    let mut factory_option = Some(&mut factory as &mut dyn Factory);
    rehydrate_script_listener_actions(&file, &mut machine, None, None, &mut factory_option)
        .expect("run the first genuine DataContext retry");
    assert!(
        machine.has_scripted_object_instance(definition),
        "the first genuine DataContext boundary must create occurrence three"
    );
}

#[test]
fn no_factory_state_change_defers_listener_until_factory_pointer_uses_latest_source_once() {
    let bytes = bound_listener_input_file(
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                nuxie.trigger("late_generator")
                return {
                    init = function(_self, _initContext)
                        nuxie.trigger("late_init")
                        return true
                    end,
                    performAction = function(self, _invocation)
                        if self.boundAmount ~= 17 then
                            error("first Factory pointer observed a stale bound source")
                        end
                        nuxie.trigger("late_perform")
                    end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import deferred listener fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build deferred listener file"));
    let (mut session, bootstrap) =
        flow_session::FlowSession::create(file, flow_session::FlowSessionConfig::default())
            .expect("create the no-Factory flow session");
    let root = bootstrap
        .catalog
        .root_instance_id
        .expect("fixture root instance");

    let no_factory = session
        .perform(flow_session::FlowOperation::StateBatch(
            flow_session::FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![flow_session::FlowStateMutation::SetValue {
                    instance: flow_session::FlowInstanceRef::Existing(root),
                    path: "amount".to_owned(),
                    value: flow_session::FlowScalarValue::Number(17.0),
                }],
                new_instances: Vec::new(),
            },
        ))
        .expect("stage the source before a File VM exists");
    assert!(
        no_factory.outputs.iter().all(|output| !matches!(
            output.payload,
            flow_session::FlowOutputPayload::HostCommand { .. }
        )),
        "an optional-Factory operation cannot manufacture a script table before the File VM exists"
    );

    let pointer = |pointer_id| {
        flow_session::FlowOperation::PointerBatch(flow_session::FlowPointerBatch {
            events: vec![flow_session::FlowPointerEvent {
                kind: flow_session::FlowPointerKind::Down,
                pointer_id,
                x: 0.0,
                y: 0.0,
                timestamp_seconds: pointer_id as f32,
            }],
        })
    };
    let mut factory = RecordingFactory::new();
    let first = session
        .perform_with_factory(pointer(1), &mut factory)
        .expect("first Factory pointer completes the deferred lifecycle");
    let second = session
        .perform_with_factory(pointer(2), &mut factory)
        .expect("second Factory pointer reuses the retained table");
    let host_names = first
        .outputs
        .iter()
        .chain(&second.outputs)
        .filter_map(|output| match &output.payload {
            flow_session::FlowOutputPayload::HostCommand { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        host_names,
        [
            "late_generator",
            "late_init",
            "late_perform",
            "late_perform"
        ],
        "the first Factory boundary materializes and initializes once, then both pointers use the retained occurrence"
    );
}

#[test]
fn first_factory_state_batch_builds_file_vm_and_applies_latest_listener_sources() {
    let bytes = bound_listener_input_file(
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                nuxie.trigger("batch_generator")
                return {
                    init = function(_self, _initContext)
                        nuxie.trigger("batch_init")
                        return true
                    end,
                    boundPulse = function(self)
                        if self.boundAmount ~= 17 then
                            error("first Factory StateBatch observed a stale bound source")
                        end
                        nuxie.trigger("batch_bound_trigger")
                    end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import cold StateBatch fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build cold StateBatch file"));
    let (mut session, bootstrap) =
        flow_session::FlowSession::create(file, flow_session::FlowSessionConfig::default())
            .expect("create the cold no-Factory session");
    let root = bootstrap
        .catalog
        .root_instance_id
        .expect("fixture root instance");
    let mut factory = RecordingFactory::new();
    let result = session
        .perform_with_factory(
            flow_session::FlowOperation::StateBatch(flow_session::FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![
                    flow_session::FlowStateMutation::SetValue {
                        instance: flow_session::FlowInstanceRef::Existing(root),
                        path: "amount".to_owned(),
                        value: flow_session::FlowScalarValue::Number(17.0),
                    },
                    flow_session::FlowStateMutation::FireTrigger {
                        instance: flow_session::FlowInstanceRef::Existing(root),
                        path: "pulse".to_owned(),
                    },
                ],
                new_instances: Vec::new(),
            }),
            &mut factory,
        )
        .expect("the first Factory StateBatch completes the File and listener lifecycle");
    let host_names = result
        .outputs
        .iter()
        .filter_map(|output| match &output.payload {
            flow_session::FlowOutputPayload::HostCommand { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        host_names,
        ["batch_generator", "batch_init", "batch_bound_trigger"],
        "File preparation precedes one listener generator/init and the same operation applies the mutated fixed sources"
    );
}

#[test]
fn first_no_factory_listener_preparation_does_not_make_an_idle_nonzero_frame_changed() {
    let (_file, mut instance, mut machine, mut factory) = prepared_machine(
        br#"
            return function(_context)
                return {
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        1,
    );
    while instance.raw_mut().update_pass() {}
    assert!(
        !instance
            .prepare_flow_scripts(&mut factory)
            .expect("prepare the File VM without initializing the listener"),
        "the listener-only fixture has no scripted artboard draw work"
    );
    assert!(!machine.scripted_object_initialization_complete());

    assert!(
        !instance.advance_with_state_machine(&mut machine, 0.25),
        "C++ derives nonzero advanceAndApply keep-going from runtime work and reports, not constructor preparation"
    );
    assert!(machine.scripted_object_initialization_complete());
}

#[test]
fn no_factory_advance_leaves_a_valid_listener_pending_until_the_first_factory_boundary() {
    let (_file, mut instance, mut machine, mut factory) = prepared_machine(
        br#"
            return function(_context)
                return {
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        1,
    );
    while instance.raw_mut().update_pass() {}
    let scripted_object_ids = machine
        .scripted_objects()
        .iter()
        .map(|definition| definition.scripted_object_global_id())
        .collect::<Vec<_>>();
    assert_eq!(scripted_object_ids.len(), 1);

    assert!(
        !instance.advance_with_state_machine(&mut machine, 0.25),
        "an optional-Factory frame cannot count deferred construction as runtime work"
    );
    assert!(!machine.scripted_object_initialization_complete());
    assert!(
        scripted_object_ids
            .iter()
            .all(|global_id| !machine.has_scripted_object_instance(*global_id)),
        "the no-Factory frame must not partially materialize the C++ constructor lifecycle"
    );

    assert!(
        !instance
            .try_advance_with_state_machine_and_factory(&mut machine, 0.25, &mut factory)
            .expect("the first Factory boundary materializes the pending occurrence"),
        "constructor preparation alone is not a nonzero-frame keep-going reason"
    );
    assert!(machine.scripted_object_initialization_complete());
    assert!(
        scripted_object_ids
            .iter()
            .all(|global_id| machine.has_scripted_object_instance(*global_id))
    );
    assert!(
        !instance
            .try_advance_with_state_machine_and_factory(&mut machine, 0.25, &mut factory)
            .expect("a later Factory boundary reuses the retained occurrence"),
        "the completed constructor lifecycle must remain idle on an unchanged frame"
    );
}

#[test]
fn foldered_listener_protocol_uses_resolved_asset_identity_not_display_name() {
    let bytes = scripted_listener_file_with_folder_and_inputs(
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                return {
                    performAction = function(_self, _invocation)
                        nuxie.trigger("foldered_listener")
                    end,
                }
            end
        "#,
        1,
        Some("interactions"),
        |_, _| {},
    );
    let file = Arc::new(
        File::from_runtime(
            read_runtime_file_for_facade(&bytes).expect("import foldered listener fixture"),
        )
        .expect("build foldered listener file"),
    );
    let mut factory = RecordingFactory::new();
    let (mut session, _) = flow_session::FlowSession::create_with_factory(
        file,
        flow_session::FlowSessionConfig::default(),
        &mut factory,
    )
    .expect("create foldered listener session");

    let result = session
        .perform_with_factory(
            flow_session::FlowOperation::PointerBatch(flow_session::FlowPointerBatch {
                events: vec![flow_session::FlowPointerEvent {
                    kind: flow_session::FlowPointerKind::Down,
                    pointer_id: 1,
                    x: 0.0,
                    y: 0.0,
                    timestamp_seconds: 0.0,
                }],
            }),
            &mut factory,
        )
        .expect("perform foldered listener");
    assert!(
        result.outputs.iter().any(|output| matches!(
            &output.payload,
            flow_session::FlowOutputPayload::HostCommand { name, .. }
                if name == "foldered_listener"
        )),
        "C++ resolves the ScriptAsset by the retained file-asset pointer, independent of folder-qualified display spelling"
    );
}

#[test]
fn concrete_scripted_child_advance_forces_zero_keep_going_without_consuming_supplied_trigger() {
    let bytes = bound_listener_input_file(
        br#"
            return function(_context)
                return {
                    boundPulse = function(_self) end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import child trigger fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build child trigger file"));
    let mut owning_artboard = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate owning artboard");
    let mut factory = RecordingFactory::new();
    owning_artboard
        .prepare_flow_scripts(&mut factory)
        .expect("prepare the File VM");
    let root = owning_artboard
        .instantiate_view_model_instance(0)
        .expect("instantiate the authored root");
    let supplied = nuxie_runtime::script_view_model_from_owned(file.runtime(), root.handle())
        .expect("project the supplied root into the script facade");
    assert!(supplied.fire_trigger("pulse"));
    assert_eq!(supplied.trigger("pulse"), Some(1));

    let mut child =
        FileScriptArtboard::new_with_view_model(Arc::clone(&file), 0, None, Some(supplied.clone()))
            .expect("instantiate the concrete scripted child");
    assert_eq!(
        supplied.trigger("pulse"),
        Some(1),
        "child construction binds the supplied root without consuming it"
    );
    assert!(
        nuxie_runtime::ScriptArtboard::advance(&mut child, 0.0)
            .expect("advance the concrete child"),
        "C++ advanceAndApply forces a zero-second child frame to keep going"
    );
    assert_eq!(
        supplied.trigger("pulse"),
        Some(1),
        "script-driven child advance resets Artboard components but leaves root ViewModel consumption to the owning host frame"
    );
    assert!(supplied.advance_script_frame());
    assert_eq!(supplied.trigger("pulse"), Some(0));
}

#[test]
fn flow_pointer_callbacks_receive_event_time_and_the_prior_delivered_position() {
    let bytes = scripted_listener_file(
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                return {
                    performAction = function(_self, invocation)
                        local pointer = invocation:asPointerEvent()
                        nuxie.trigger("pointer_payload", {
                            x = pointer.position.x,
                            y = pointer.position.y,
                            previousX = pointer.previousPosition.x,
                            previousY = pointer.previousPosition.y,
                            timeStamp = pointer.timeStamp,
                        })
                    end,
                }
            end
        "#,
        1,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import pointer callback fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build pointer callback file"));
    let mut factory = RecordingFactory::new();
    let (mut session, _) = flow_session::FlowSession::create_with_factory(
        file,
        flow_session::FlowSessionConfig::default(),
        &mut factory,
    )
    .expect("create pointer callback session");

    let result = session
        .perform_with_factory(
            flow_session::FlowOperation::PointerBatch(flow_session::FlowPointerBatch {
                events: vec![
                    flow_session::FlowPointerEvent {
                        kind: flow_session::FlowPointerKind::Down,
                        pointer_id: 1,
                        x: 10.0,
                        y: 20.0,
                        timestamp_seconds: 1.25,
                    },
                    flow_session::FlowPointerEvent {
                        kind: flow_session::FlowPointerKind::Down,
                        pointer_id: 1,
                        x: 30.0,
                        y: 40.0,
                        timestamp_seconds: 2.5,
                    },
                ],
            }),
            &mut factory,
        )
        .expect("perform successive pointer callbacks");

    let payloads = result
        .outputs
        .iter()
        .filter_map(|output| match &output.payload {
            flow_session::FlowOutputPayload::HostCommand { name, payload }
                if name == "pointer_payload" =>
            {
                Some(payload.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let payload = |x, y, previous_x, previous_y, timestamp| {
        flow_session::FlowHostValue::Object(std::collections::BTreeMap::from([
            (
                "previousX".to_owned(),
                flow_session::FlowHostValue::Number(previous_x),
            ),
            (
                "previousY".to_owned(),
                flow_session::FlowHostValue::Number(previous_y),
            ),
            (
                "timeStamp".to_owned(),
                flow_session::FlowHostValue::Number(timestamp),
            ),
            ("x".to_owned(), flow_session::FlowHostValue::Number(x)),
            ("y".to_owned(), flow_session::FlowHostValue::Number(y)),
        ]))
    };
    assert_eq!(
        payloads,
        [
            payload(10.0, 20.0, 10.0, 20.0, 1.25),
            payload(30.0, 40.0, 10.0, 20.0, 2.5),
        ]
    );
}

#[test]
fn listener_authored_inputs_are_hydrated_before_init() {
    let bytes = scripted_listener_file_with_inputs(
        br#"
            local nuxie = require("nuxie")

            return function(context)
                return {
                    pulse = function(_self)
                        error("an authored trigger must not fire during initial hydration")
                    end,
                    init = function(self, initContext)
                        if context ~= initContext then
                            error("init received a different Context")
                        end
                        if initContext:viewModel() ~= nil then
                            error("fixture unexpectedly has a root view model")
                        end
                        if self.enabled ~= true then error("boolean input was not hydrated") end
                        if self.amount ~= 42.5 then error("number input was not hydrated") end
                        if self.tint ~= 287454020 then error("color input was not hydrated") end
                        if self.label ~= "ready" then error("string input was not hydrated") end
                        if self.panel == nil or self.panel.width ~= 100 then
                            error("artboard input was not hydrated")
                        end
                        nuxie.trigger("authored_inputs_ready")
                        return true
                    end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        1,
        |_, bytes| {
            push_object(bytes, "ScriptInputBoolean", |bytes| {
                push_string(bytes, "ScriptInputBoolean", "name", "enabled");
                push_uint(bytes, "ScriptInputBoolean", "propertyValue", 1);
            });
            push_object(bytes, "ScriptInputNumber", |bytes| {
                push_string(bytes, "ScriptInputNumber", "name", "amount");
                push_f32(bytes, "ScriptInputNumber", "propertyValue", 42.5);
            });
            push_object(bytes, "ScriptInputColor", |bytes| {
                push_string(bytes, "ScriptInputColor", "name", "tint");
                push_color(bytes, "ScriptInputColor", "propertyValue", 0x1122_3344);
            });
            push_object(bytes, "ScriptInputString", |bytes| {
                push_string(bytes, "ScriptInputString", "name", "label");
                push_string(bytes, "ScriptInputString", "propertyValue", "ready");
            });
            push_object(bytes, "ScriptInputTrigger", |bytes| {
                push_string(bytes, "ScriptInputTrigger", "name", "pulse");
            });
            push_object(bytes, "ScriptInputArtboard", |bytes| {
                push_string(bytes, "ScriptInputArtboard", "name", "panel");
                push_uint(bytes, "ScriptInputArtboard", "artboardId", 0);
            });
        },
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import authored-input fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build authored-input file"));
    let mut factory = RecordingFactory::new();
    let (_session, creation) = flow_session::FlowSession::create_with_factory(
        file,
        flow_session::FlowSessionConfig::default(),
        &mut factory,
    )
    .expect("create authored-input flow session");

    assert!(creation.outputs.iter().any(|output| matches!(
        &output.payload,
        flow_session::FlowOutputPayload::HostCommand { name, .. }
            if name == "authored_inputs_ready"
    )));
}

#[test]
fn listener_bound_context_inputs_rehydrate_and_fire_trigger_edges() {
    let bytes = bound_listener_input_file(
        br#"
        local nuxie = require("nuxie")

        return function(context)
            local function verify(self, contextAmount, boundAmount)
                local root = context:viewModel()
                if root == nil or root.amount.value ~= contextAmount then
                    error("Context.viewModel is stale")
                end
                if self.boundAmount ~= boundAmount then
                    error("bound scalar is stale")
                end
                if self.boundChild == nil or self.boundChild.score.value ~= 4 then
                    error("bound nested view model is missing")
                end
            end
            return {
                init = function(self, initContext)
                    if context ~= initContext then error("init Context changed") end
                    -- C++ cloneScriptedObject/reinit happens before
                    -- updateDataBinds(false), so init sees the cloned authored
                    -- scalar while Context already exposes the live root.
                    verify(self, 9.5, 1)
                    nuxie.trigger("bound_init_ready")
                    return true
                end,
                boundPulse = function(self)
                    verify(self, 17, 17)
                    nuxie.trigger("bound_trigger_fired")
                end,
                performAction = function(self, _invocation)
                    verify(self, 17, 17)
                end,
            }
        end
    "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import bound-input fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build bound-input file"));
    let mut factory = RecordingFactory::new();
    let (mut session, creation) = flow_session::FlowSession::create_with_factory(
        file,
        flow_session::FlowSessionConfig::default(),
        &mut factory,
    )
    .expect("create bound-input flow session");
    assert!(creation.outputs.iter().any(|output| matches!(
        &output.payload,
        flow_session::FlowOutputPayload::HostCommand { name, .. }
            if name == "bound_init_ready"
    )));

    let root = creation
        .bootstrap
        .catalog
        .root_instance_id
        .expect("fixture root instance");
    let result = session
        .perform_with_factory(
            flow_session::FlowOperation::StateBatch(flow_session::FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![
                    flow_session::FlowStateMutation::SetValue {
                        instance: flow_session::FlowInstanceRef::Existing(root),
                        path: "amount".to_owned(),
                        value: flow_session::FlowScalarValue::Number(17.0),
                    },
                    flow_session::FlowStateMutation::FireTrigger {
                        instance: flow_session::FlowInstanceRef::Existing(root),
                        path: "pulse".to_owned(),
                    },
                ],
                new_instances: Vec::new(),
            }),
            &mut factory,
        )
        .expect("rebind listener inputs after state batch");
    assert!(result.outputs.iter().any(|output| matches!(
        &output.payload,
        flow_session::FlowOutputPayload::HostCommand { name, .. }
            if name == "bound_trigger_fired"
    )));
}

#[test]
fn repeated_bound_trigger_edges_reset_and_invoke_once_per_frame() {
    let bytes = bound_listener_input_file(
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                return {
                    __nuxieProbeCount = 0,
                    init = function(_self, _initContext)
                        return true
                    end,
                    boundPulse = function(self)
                        self.__nuxieProbeCount += 1
                        nuxie.trigger("repeat_trigger_" .. self.__nuxieProbeCount)
                    end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
    );

    let runtime = read_runtime_file_for_facade(&bytes).expect("import repeated-trigger fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build repeated-trigger file"));
    let mut factory = RecordingFactory::new();
    let (mut session, creation) = flow_session::FlowSession::create_with_factory(
        file,
        flow_session::FlowSessionConfig::default(),
        &mut factory,
    )
    .expect("create repeated-trigger flow session");
    let root = creation
        .bootstrap
        .catalog
        .root_instance_id
        .expect("repeated-trigger root instance");

    for (index, expected) in ["repeat_trigger_1", "repeat_trigger_2"]
        .into_iter()
        .enumerate()
    {
        let result = session
            .perform_with_factory(
                flow_session::FlowOperation::StateBatch(flow_session::FlowStateBatch {
                    host_mutation_id: None,
                    mutations: vec![flow_session::FlowStateMutation::FireTrigger {
                        instance: flow_session::FlowInstanceRef::Existing(root),
                        path: "pulse".to_owned(),
                    }],
                    new_instances: Vec::new(),
                }),
                &mut factory,
            )
            .expect("fire repeated bound trigger");
        assert!(
            result.outputs.iter().any(|output| matches!(
                &output.payload,
                flow_session::FlowOutputPayload::HostCommand { name, .. }
                    if name == expected
            )),
            "Rust must invoke the retained ScriptInputTrigger callback for {expected}; outputs: {:?}",
            result.outputs,
        );
        if index == 0 {
            // Pinned C++ resets the source under SuppressDelegation and the
            // ScriptInputTrigger target through the next apply pass
            // (`viewmodel_instance_trigger.cpp:20-27`;
            // `script_input_trigger.cpp:60-67`). The paired live probe test
            // locks the target sequence to 0→1→0→1.
            let reset = session
                .perform_with_factory(
                    flow_session::FlowOperation::Advance(flow_session::FlowAdvance {
                        timestamp_seconds: 0.0,
                        delta_seconds: 0.0,
                        render: false,
                    }),
                    &mut factory,
                )
                .expect("advance-and-apply resets the first trigger edge");
            assert!(
                !reset.outputs.iter().any(|output| matches!(
                    &output.payload,
                    flow_session::FlowOutputPayload::HostCommand { name, .. }
                        if name.starts_with("repeat_trigger_")
                )),
                "the suppressed 1→0 reset must not invoke the script callback"
            );
        }
    }
}

#[test]
fn scripted_listener_name_based_bind_uses_the_persistent_manifest_resolver() {
    let bytes = bound_listener_input_file_with_name_based_amount(
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                return {
                    init = function(_self, _initContext) return true end,
                    boundPulse = function(self)
                        if self.boundAmount == 9.5 then
                            nuxie.trigger("manifest_listener_initial")
                        elseif self.boundAmount == 17 then
                            nuxie.trigger("manifest_listener_live")
                        else
                            error("name-based listener input did not resolve through ManifestAsset")
                        end
                    end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import manifest listener fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build manifest listener file"));
    let mut factory = RecordingFactory::new();
    let (mut session, creation) = flow_session::FlowSession::create_with_factory(
        file,
        flow_session::FlowSessionConfig::default(),
        &mut factory,
    )
    .expect("create manifest listener flow session");
    let root = creation
        .bootstrap
        .catalog
        .root_instance_id
        .expect("manifest listener root");

    let initial = session
        .perform_with_factory(
            flow_session::FlowOperation::StateBatch(flow_session::FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![flow_session::FlowStateMutation::FireTrigger {
                    instance: flow_session::FlowInstanceRef::Existing(root),
                    path: "pulse".to_owned(),
                }],
                new_instances: Vec::new(),
            }),
            &mut factory,
        )
        .expect("fire listener against the initial name-based value");
    assert!(initial.outputs.iter().any(|output| matches!(
        &output.payload,
        flow_session::FlowOutputPayload::HostCommand { name, .. }
            if name == "manifest_listener_initial"
    )));

    let live = session
        .perform_with_factory(
            flow_session::FlowOperation::StateBatch(flow_session::FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![
                    flow_session::FlowStateMutation::SetValue {
                        instance: flow_session::FlowInstanceRef::Existing(root),
                        path: "amount".to_owned(),
                        value: flow_session::FlowScalarValue::Number(17.0),
                    },
                    flow_session::FlowStateMutation::FireTrigger {
                        instance: flow_session::FlowInstanceRef::Existing(root),
                        path: "pulse".to_owned(),
                    },
                ],
                new_instances: Vec::new(),
            }),
            &mut factory,
        )
        .expect("update and refire the name-based listener input");
    assert!(live.outputs.iter().any(|output| matches!(
        &output.payload,
        flow_session::FlowOutputPayload::HostCommand { name, .. }
            if name == "manifest_listener_live"
    )));
}

#[test]
fn scripted_converter_custom_inputs_hydrate_before_init_then_bind_before_convert() {
    let bytes = bound_listener_scripted_converter_file(
        br#"
            return function(_context)
                return {
                    init = function(_self, _initContext)
                        return true
                    end,
                    performAction = function(_self, _invocation)
                    end,
                }
            end
        "#,
        br#"
            local nuxie = require("nuxie")

            return function(context)
                if context:viewModel() == nil or context:viewModel().amount.value ~= 9.5 then
                    error("converter generator did not receive the live DataContext")
                end
                nuxie.trigger("converter_generator_context")
                return {
                    init = function(self, _initContext)
                        -- The Backboard converter definition is cold metadata;
                        -- this assertion belongs to the concrete clone owned by
                        -- the listener DataBind occurrence.
                        if self.customAmount == nil then
                            return true
                        end
                        if self.customAmount ~= 1 then
                            error("converter init did not see the authored cloned input")
                        end
                        nuxie.trigger("converter_init")
                        return true
                    end,
                    convert = function(self, input)
                        if self.customAmount ~= 9.5 then
                            error("converter custom input was not updated before conversion")
                        end
                        -- A later bindFromContext must rehydrate this same
                        -- persistent table from the cloned Core target before
                        -- conversion, even when the source value is unchanged.
                        self.customAmount = -100
                        nuxie.trigger("converter_convert")
                        return input
                    end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import converted-listener fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build converted-listener file"));
    let mut factory = RecordingFactory::new();
    let (mut session, creation) = flow_session::FlowSession::create_with_factory(
        file,
        flow_session::FlowSessionConfig::default(),
        &mut factory,
    )
    .expect("create converted-listener flow session");

    let mut commands = creation
        .outputs
        .iter()
        .filter_map(|output| match &output.payload {
            flow_session::FlowOutputPayload::HostCommand { name, .. }
                if name == "converter_generator_context"
                    || name == "converter_init"
                    || name == "converter_convert" =>
            {
                Some(name.as_str())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let root = creation
        .bootstrap
        .catalog
        .root_instance_id
        .expect("fixture root instance");
    let advance = session
        .perform_with_factory(
            flow_session::FlowOperation::Advance(flow_session::FlowAdvance {
                timestamp_seconds: 0.0,
                delta_seconds: 0.0,
                render: false,
            }),
            &mut factory,
        )
        .expect("apply the converter's first live source update");
    commands.extend(
        advance
            .outputs
            .iter()
            .filter_map(|output| match &output.payload {
                flow_session::FlowOutputPayload::HostCommand { name, .. }
                    if name == "converter_generator_context"
                        || name == "converter_init"
                        || name == "converter_convert" =>
                {
                    Some(name.as_str())
                }
                _ => None,
            }),
    );
    let steady = session
        .perform_with_factory(
            flow_session::FlowOperation::Advance(flow_session::FlowAdvance {
                timestamp_seconds: 1.0,
                delta_seconds: 0.0,
                render: false,
            }),
            &mut factory,
        )
        .expect("advance an unchanged bound converter occurrence");
    commands.extend(
        steady
            .outputs
            .iter()
            .filter_map(|output| match &output.payload {
                flow_session::FlowOutputPayload::HostCommand { name, .. }
                    if name == "converter_generator_context"
                        || name == "converter_init"
                        || name == "converter_convert" =>
                {
                    Some(name.as_str())
                }
                _ => None,
            }),
    );
    let rebind = session
        .perform_with_factory(
            flow_session::FlowOperation::StateBatch(flow_session::FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![flow_session::FlowStateMutation::SetValue {
                    instance: flow_session::FlowInstanceRef::Existing(root),
                    path: "amount".to_owned(),
                    value: flow_session::FlowScalarValue::Number(9.5),
                }],
                new_instances: Vec::new(),
            }),
            &mut factory,
        )
        .expect("rehydrate the existing converter table on an unchanged-source rebind");
    commands.extend(
        rebind
            .outputs
            .iter()
            .filter_map(|output| match &output.payload {
                flow_session::FlowOutputPayload::HostCommand { name, .. }
                    if name == "converter_generator_context"
                        || name == "converter_init"
                        || name == "converter_convert" =>
                {
                    Some(name.as_str())
                }
                _ => None,
            }),
    );
    assert_eq!(
        commands,
        [
            "converter_generator_context",
            "converter_init",
            "converter_convert",
            "converter_convert",
        ],
        "C++ creates the converter against its live DataContext, does not rebind an unchanged ordinary frame, then rehydrates the same table at the explicit context-rebind boundary"
    );
}

#[test]
fn scripted_converter_name_based_child_bind_uses_the_persistent_manifest_resolver() {
    let bytes = bound_listener_scripted_converter_file_with_name_based_custom_amount(
        br#"
            return function(_context)
                return {
                    init = function(_self, _initContext) return true end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                return {
                    init = function(self, _initContext)
                        if self.customAmount ~= 1 then
                            error("converter did not retain its authored cold input")
                        end
                        return true
                    end,
                    convert = function(self, input)
                        if self.customAmount == 9.5 then
                            nuxie.trigger("manifest_converter_initial")
                        elseif self.customAmount == 17 then
                            nuxie.trigger("manifest_converter_live")
                        else
                            error("name-based converter input did not resolve through ManifestAsset")
                        end
                        return input
                    end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import manifest converter fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build manifest converter file"));
    let mut factory = RecordingFactory::new();
    let (mut session, creation) = flow_session::FlowSession::create_with_factory(
        file,
        flow_session::FlowSessionConfig::default(),
        &mut factory,
    )
    .expect("create manifest converter flow session");
    let root = creation
        .bootstrap
        .catalog
        .root_instance_id
        .expect("manifest converter root");

    let initial = session
        .perform_with_factory(
            flow_session::FlowOperation::Advance(flow_session::FlowAdvance {
                timestamp_seconds: 0.0,
                delta_seconds: 0.0,
                render: false,
            }),
            &mut factory,
        )
        .expect("apply the initial name-based converter source");
    assert!(initial.outputs.iter().any(|output| matches!(
        &output.payload,
        flow_session::FlowOutputPayload::HostCommand { name, .. }
            if name == "manifest_converter_initial"
    )));

    let live = session
        .perform_with_factory(
            flow_session::FlowOperation::StateBatch(flow_session::FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![flow_session::FlowStateMutation::SetValue {
                    instance: flow_session::FlowInstanceRef::Existing(root),
                    path: "amount".to_owned(),
                    value: flow_session::FlowScalarValue::Number(17.0),
                }],
                new_instances: Vec::new(),
            }),
            &mut factory,
        )
        .expect("update the name-based converter source");
    assert!(live.outputs.iter().any(|output| matches!(
        &output.payload,
        flow_session::FlowOutputPayload::HostCommand { name, .. }
            if name == "manifest_converter_live"
    )));
}

#[test]
fn public_factory_advance_mounts_scripted_listener_converter_once() {
    let bytes = bound_listener_scripted_converter_file(
        br#"
            return function(_context)
                return {
                    init = function(_self, _initContext) return true end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        br#"
            local nuxie = require("nuxie")

            return function(context)
                if context:viewModel() == nil or context:viewModel().amount.value ~= 9.5 then
                    error("public advance did not supply the live DataContext")
                end
                nuxie.trigger("public_converter_generator")
                return {
                    init = function(self, _initContext)
                        if self.customAmount ~= 1.0 then
                            error("public advance did not hydrate the authored converter input")
                        end
                        nuxie.trigger("public_converter_init")
                        return true
                    end,
                    convert = function(self, input)
                        if self.customAmount ~= 9.5 then
                            error("public advance converted before the bound input update")
                        end
                        nuxie.trigger("public_converter_convert")
                        return input
                    end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import public-advance fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build public-advance file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate state machine");
    let mut root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate default view model");
    let mut factory = RecordingFactory::new();

    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.0,
            &mut root,
            &mut factory,
        )
        .expect("public factory advance");
    let first = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name.starts_with("public_converter_") => {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        first,
        [
            "public_converter_generator",
            "public_converter_init",
            "public_converter_convert",
        ],
        "the public factory path must complete the same ScriptedListenerAction converter lifecycle as FlowSession"
    );

    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.0,
            &mut root,
            &mut factory,
        )
        .expect("steady public factory advance");
    assert!(
        instance
            .drain_flow_host_commands()
            .into_iter()
            .all(|command| !matches!(
                command,
                LuaHostCommand::Trigger { ref name, .. }
                    if name == "public_converter_generator"
                        || name == "public_converter_init"
            )),
        "an unchanged public frame must not regenerate or reinitialize the retained occurrence"
    );
}

#[test]
fn imported_scripted_transition_conditions_mount_and_evaluate_without_manual_injection() {
    let fixture = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
        .join("tests/unit_tests/assets/scripted_transition_condition.riv");
    let bytes = std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("missing fixture {}: {error}", fixture.display()));
    let file = Arc::new(
        File::import_with_unsigned_scripts(&bytes)
            .expect("import scripted transition-condition fixture"),
    );
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate state machine");
    let mut root = instance
        .instantiate_view_model()
        .expect("instantiate scripted transition view model");
    let condition_globals = machine
        .scripted_objects()
        .iter()
        .filter(|definition| {
            definition.scripted_object_kind()
                == nuxie_runtime::ScriptedStateMachineObjectKind::TransitionCondition
        })
        .map(|definition| definition.scripted_object_global_id())
        .collect::<Vec<_>>();
    assert_eq!(condition_globals.len(), 2);
    assert!(
        condition_globals
            .iter()
            .all(|global_id| !machine.has_scripted_object_instance(*global_id))
    );

    let mut factory = RecordingFactory::new();
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.1,
            &mut root,
            &mut factory,
        )
        .expect("cold/live scripted-object lifecycle");
    assert!(
        condition_globals
            .iter()
            .all(|global_id| machine.has_scripted_object_instance(*global_id)),
        "every imported ScriptedTransitionCondition owns a fresh live table"
    );

    assert!(root.set_bool("timelineBool", true));
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.016,
            &mut root,
            &mut factory,
        )
        .expect("timeline condition evaluates");
    assert!(
        machine.changed_state_count() > 0,
        "the imported scripted timeline condition must change state"
    );

    assert!(root.set_bool("anyStateBool", true));
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.016,
            &mut root,
            &mut factory,
        )
        .expect("any-state condition evaluates");
    assert!(
        machine.changed_state_count() > 0,
        "the imported scripted any-state condition must change state"
    );
}

#[test]
fn public_factory_advance_mounts_each_ordinary_converter_occurrence_once() {
    let bytes = ordinary_state_machine_scripted_converter_file(
        br#"
            local nuxie = require("nuxie")
            local generation = 0

            return function(context)
                if context:viewModel() == nil or context:viewModel().amount.value ~= 9.5 then
                    error("ordinary converter did not receive the live DataContext")
                end
                generation += 1
                local occurrence = generation
                nuxie.trigger("ordinary_generator_" .. occurrence)
                return {
                    init = function(self, _initContext)
                        if self.customAmount ~= 1.0 then
                            error("ordinary converter init did not see its cloned authored input")
                        end
                        nuxie.trigger("ordinary_init_" .. occurrence)
                        return true
                    end,
                    convert = function(self, input)
                        nuxie.trigger("ordinary_convert_enter_" .. occurrence)
                        if self.customAmount ~= 9.5 then
                            error("ordinary converter ran before its custom input update")
                        end
                        nuxie.trigger("ordinary_convert_" .. occurrence)
                        return input
                    end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import ordinary-converter fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build ordinary-converter file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate state machine");
    let cold = machine.scripted_data_converter_occurrence_snapshots();
    assert_eq!(cold.len(), 2);
    assert!(cold.iter().all(|occurrence| !occurrence.attached));
    assert_eq!(
        cold.iter()
            .map(|occurrence| occurrence.parent_data_bind_index)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(
        cold[0].converter_global_id, cold[1].converter_global_id,
        "both DataBinds intentionally reference one converter definition"
    );

    let mut root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate default view model");
    let mut factory = RecordingFactory::new();
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.0,
            &mut root,
            &mut factory,
        )
        .expect("public ordinary-converter advance");
    assert!(
        machine
            .scripted_data_converter_occurrence_snapshots()
            .iter()
            .all(|occurrence| occurrence.attached),
        "each concrete ordinary DataBind occurrence owns a live table"
    );
    let first = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. }
                if name.starts_with("ordinary_generator_")
                    || name.starts_with("ordinary_init_")
                    || name.starts_with("ordinary_convert_enter_")
                    || name.starts_with("ordinary_convert_") =>
            {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        first,
        [
            "ordinary_generator_1",
            "ordinary_init_1",
            "ordinary_generator_2",
            "ordinary_init_2",
            "ordinary_convert_enter_1",
            "ordinary_convert_1",
            "ordinary_convert_enter_2",
            "ordinary_convert_2",
        ],
        "C++ binds, constructs, hydrates, and initializes each repeated occurrence in authored order before the fixed DataBind container updates them"
    );

    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.0,
            &mut root,
            &mut factory,
        )
        .expect("steady ordinary-converter advance");
    assert!(
        instance
            .drain_flow_host_commands()
            .into_iter()
            .all(|command| !matches!(
                command,
                LuaHostCommand::Trigger { ref name, .. }
                    if name.starts_with("ordinary_generator_")
                        || name.starts_with("ordinary_init_")
            )),
        "an unchanged frame retains both occurrence tables"
    );
}

#[test]
fn ordinary_converter_replacement_is_visible_to_the_next_authored_bind_same_frame() {
    let bytes = ordinary_scripted_converter_replaces_nested_source_file(
        br#"
            local nuxie = require("nuxie")

            return function(context)
                nuxie.trigger("replacement_generator")
                return {
                    init = function(_self, _initContext)
                        nuxie.trigger("replacement_init_enter")
                        context:viewModel().child.value = Data.Child.new("child-new")
                        if context:viewModel().child.value.leaf.value ~= 22 then
                            error("replacement was not visible through the live context")
                        end
                        nuxie.trigger("replacement_init_exit")
                        return true
                    end,
                    convert = function(_self, input) return input end,
                }
            end
        "#,
    );
    let runtime =
        read_runtime_file_for_facade(&bytes).expect("import nested replacement ordering fixture");
    let file =
        Arc::new(File::from_runtime(runtime).expect("build nested replacement ordering file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate nested replacement ordering artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate nested replacement ordering machine");
    let mut root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate authored root");
    let probe_root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate replacement API control root");
    let script_probe_root =
        nuxie_runtime::script_view_model_from_owned(file.runtime(), probe_root.handle())
            .expect("project the replacement API control root");
    let script_replacement = nuxie_runtime::script_view_models(file.runtime())
        .get("Child")
        .and_then(|model| model.named_instance(Some("child-new")))
        .expect("instantiate the named replacement control");
    assert!(
        script_probe_root.set_view_model("child", &script_replacement),
        "the fixture's replacement schemas and retained identities are valid"
    );
    assert_eq!(
        root.handle()
            .linked_view_model_by_property_name_path("child")
            .and_then(|child| child.borrow().number_value_by_property_name("leaf")),
        Some(11.0),
        "the fixture begins with the old nested occurrence"
    );

    let mut factory = RecordingFactory::new();
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.0,
            &mut root,
            &mut factory,
        )
        .expect("advance the ordered converter replacement fixture");

    assert!(
        machine
            .scripted_data_converter_occurrence_snapshots()
            .iter()
            .all(|occurrence| occurrence.attached),
        "the authored converter occurrence must own its generated table"
    );
    assert_eq!(
        instance
            .drain_flow_host_commands()
            .into_iter()
            .filter_map(|command| match command {
                LuaHostCommand::Trigger { name, .. } if name.starts_with("replacement_") => {
                    Some(name)
                }
                _ => None,
            })
            .collect::<Vec<_>>(),
        [
            "replacement_generator",
            "replacement_init_enter",
            "replacement_init_exit",
        ],
        "the generated occurrence must complete its replacement before the next bind"
    );
    assert_eq!(
        root.handle()
            .linked_view_model_by_property_name_path("child")
            .and_then(|child| child.borrow().number_value_by_property_name("leaf")),
        Some(22.0),
        "the converter init replaces the live nested occurrence"
    );
    assert_eq!(
        machine.bindable_number_value_for_data_bind(0),
        Some(5.0),
        "the converter-owned outer bind still resolves its authored kick source"
    );
    assert_eq!(
        machine.bindable_number_value_for_data_bind(1),
        Some(22.0),
        "the next authored outer bind must resolve after the preceding converter init"
    );
}

#[test]
fn public_factory_new_root_rehydrates_the_retained_ordinary_converter_same_frame() {
    let bytes = ordinary_state_machine_scripted_converter_file(
        br#"
            local nuxie = require("nuxie")
            local generation = 0

            return function(context)
                generation += 1
                local occurrence = generation
                nuxie.trigger("root_rebind_generator_" .. occurrence)
                return {
                    init = function(self, _initContext)
                        nuxie.trigger("root_rebind_init_" .. occurrence)
                        return true
                    end,
                    convert = function(self, input)
                        if self.customAmount == 9.5
                            and context:viewModel() ~= nil
                            and context:viewModel().amount.value == 9.5
                        then
                            nuxie.trigger("root_rebind_first_" .. occurrence)
                        elseif self.customAmount == 17
                            and context:viewModel() ~= nil
                            and context:viewModel().amount.value == 17
                        then
                            nuxie.trigger("root_rebind_second_" .. occurrence)
                        else
                            error("converter retained a stale DataContext or ScriptInput")
                        end
                        return input
                    end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import root-rebind fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build root-rebind file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate state machine");
    let mut first_root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate first root");
    let mut second_root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate second root");
    assert!(second_root.set_number("amount", 17.0));
    let mut factory = RecordingFactory::new();

    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.0,
            &mut first_root,
            &mut factory,
        )
        .expect("mount the first-root occurrence");
    let first = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name.starts_with("root_rebind_") => Some(name),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        first,
        [
            "root_rebind_generator_1",
            "root_rebind_init_1",
            "root_rebind_generator_2",
            "root_rebind_init_2",
            "root_rebind_first_1",
            "root_rebind_first_2",
        ],
        "each repeated outer DataBind owns its own table on the first root"
    );

    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.0,
            &mut second_root,
            &mut factory,
        )
        .expect("rebind the retained occurrences to the second root");
    let rebound = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name.starts_with("root_rebind_") => Some(name),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rebound,
        ["root_rebind_second_1", "root_rebind_second_2"],
        "C++ rehydrates each retained table against the new live DataContext before same-frame conversion, without regenerating or rerunning user init (`state_machine_instance.cpp:2901-2913`; `scripted_data_converter.cpp:170-188`)"
    );
}

#[test]
fn hydrated_viewmodel_input_retains_the_concrete_child_until_rehydration() {
    let bytes = bound_listener_input_file(
        br#"
            return function(_context)
                return {
                    init = function(_self, _initContext) return true end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import concrete-child fixture");
    let root = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(&runtime, 0, 0)
            .expect("instantiate authored root"),
    );
    let context = nuxie_runtime::RuntimeOwnedViewModelContextHandle::root(&runtime, root.clone());
    let input = (0..runtime.object_count())
        .filter_map(|index| runtime.object(index))
        .find(|object| object.type_name == "ScriptInputViewModelProperty")
        .expect("bound ViewModel script input");

    let old_child = root
        .linked_view_model_by_property_name_path("child")
        .expect("authored child A");
    let hydrated =
        nuxie_runtime::bound_script_view_model_from_owned_context(&runtime, &context, input)
            .expect("hydrate child A");
    assert_eq!(hydrated.number("score"), Some(4.0));

    let replacement = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(&runtime, 1, 0).expect("instantiate child B"),
    );
    assert!(
        replacement
            .borrow_mut()
            .set_number_by_property_name("score", 22.0)
    );
    assert!(
        root.link_view_model_by_property_name_path("child", &replacement)
            .expect("replace child A with child B")
    );

    assert_eq!(
        hydrated.number("score"),
        Some(4.0),
        "C++ ScriptedObject::setViewModelInput retains the concrete child rcp, not a parent path that follows later replacement"
    );
    assert!(hydrated.set_number("score", 41.0));
    assert_eq!(
        old_child.borrow().number_value_by_property_name("score"),
        Some(41.0)
    );
    assert_eq!(
        replacement.borrow().number_value_by_property_name("score"),
        Some(22.0)
    );

    let rehydrated =
        nuxie_runtime::bound_script_view_model_from_owned_context(&runtime, &context, input)
            .expect("explicitly rehydrate child B");
    assert_eq!(
        rehydrated.number("score"),
        Some(22.0),
        "the next explicit hydration boundary installs the replacement child"
    );
}

#[test]
fn direct_runtime_callbacks_wait_for_same_root_structural_rebind() {
    let bytes = bound_listener_input_file(
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                return {
                    init = function(_self, _initContext) return true end,
                    performAction = function(self, _invocation)
                        if self.boundChild == nil then
                            error("bound child was unavailable")
                        end
                        self.boundChild.score.value = 99
                        nuxie.trigger("bound_child_written")
                    end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import structural-rebind fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build structural-rebind file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate structural-rebind artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate structural-rebind machine");
    let mut root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate structural-rebind root");
    let old_child = root
        .handle()
        .linked_view_model_by_property_name_path("child")
        .expect("fixture old child");
    let mut factory = RecordingFactory::new();
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.0,
            &mut root,
            &mut factory,
        )
        .expect("complete the initial scripted-object lifecycle");

    let replacement = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(file.runtime(), 1, 0)
            .expect("instantiate same-valued replacement child"),
    );
    assert_eq!(
        old_child.borrow().number_value_by_property_name("score"),
        Some(4.0)
    );
    assert_eq!(
        replacement.borrow().number_value_by_property_name("score"),
        Some(4.0)
    );
    assert!(
        root.handle()
            .link_view_model_by_property_name_path("child", &replacement)
            .expect("replace the retained child")
    );

    assert!(
        !machine.pointer_down(instance.raw_mut(), 0.0, 0.0, 1),
        "Rust's split facade must not expose the stale pre-rebind ScriptInput occurrence"
    );
    assert_eq!(
        old_child.borrow().number_value_by_property_name("score"),
        Some(4.0)
    );
    assert_eq!(
        replacement.borrow().number_value_by_property_name("score"),
        Some(4.0)
    );

    try_prepare_state_machine_scripted_data_context_without_factory(
        &file,
        instance.raw(),
        &mut machine,
        Some(&root),
    )
    .expect("complete the source-corresponding retained rebind");
    machine
        .apply_scripted_listener_action_source_updates(instance.raw(), None, &mut NoopScriptHost)
        .expect("apply the rebound fixed sources");
    assert!(machine.pointer_down(instance.raw_mut(), 0.0, 0.0, 2));
    assert_eq!(
        old_child.borrow().number_value_by_property_name("score"),
        Some(4.0),
        "the detached source occurrence stays inert"
    );
    assert_eq!(
        replacement.borrow().number_value_by_property_name("score"),
        Some(99.0),
        "the rebound occurrence owns the next callback"
    );
}

#[test]
fn scripted_converter_init_bit_disabled_skips_live_init_but_still_converts() {
    const DATA_CONVERT: u64 = 1 << 10;

    let bytes = bound_listener_scripted_converter_file_with_options(
        br#"
            return function(_context)
                return {
                    init = function(_self, _initContext) return true end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                return {
                    init = function(_self, _initContext)
                        nuxie.trigger("disabled_converter_init")
                        error("the authored ScriptAsset does not implement init")
                    end,
                    convert = function(_self, input)
                        nuxie.trigger("enabled_converter_convert")
                        return input
                    end,
                }
            end
        "#,
        false,
        Some(DATA_CONVERT),
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import method-mask fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build method-mask file"));
    let mut factory = RecordingFactory::new();
    let (mut session, creation) = flow_session::FlowSession::create_with_factory(
        file,
        flow_session::FlowSessionConfig::default(),
        &mut factory,
    )
    .expect("create method-mask flow session");
    let advance = session
        .perform_with_factory(
            flow_session::FlowOperation::Advance(flow_session::FlowAdvance {
                timestamp_seconds: 0.0,
                delta_seconds: 0.0,
                render: false,
            }),
            &mut factory,
        )
        .expect("apply the first converter source update");
    let commands = creation
        .outputs
        .iter()
        .chain(&advance.outputs)
        .filter_map(|output| match &output.payload {
            flow_session::FlowOutputPayload::HostCommand { name, .. }
                if name == "disabled_converter_init" || name == "enabled_converter_convert" =>
            {
                Some(name.as_str())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        commands,
        ["enabled_converter_convert"],
        "ScriptAsset::inits gates the live method even when Lua exposes a function, while the independent convert bit remains enabled (`script_asset.cpp:145-161`; `scripted_object.cpp:427-435`)"
    );
}

#[test]
fn scripted_converter_failed_init_recreates_only_at_the_next_explicit_rebind() {
    let bytes = bound_listener_scripted_converter_file(
        br#"
            return function(_context)
                return {
                    init = function(_self, _initContext) return true end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        br#"
            local nuxie = require("nuxie")
            local generation = 0

            return function(context)
                if context:viewModel() == nil then
                    error("converter requires the live DataContext")
                end
                generation += 1
                local occurrence = generation
                if occurrence == 1 then
                    nuxie.trigger("converter_retry_generated_1")
                else
                    nuxie.trigger("converter_retry_generated_2")
                end
                return {
                    init = function(_self, _initContext)
                        if occurrence == 1 then
                            nuxie.trigger("converter_retry_init_1")
                            return false
                        end
                        nuxie.trigger("converter_retry_init_2")
                        return true
                    end,
                    convert = function(_self, input)
                        if occurrence == 1 then
                            nuxie.trigger("converter_retry_convert_1")
                        else
                            nuxie.trigger("converter_retry_convert_2")
                        end
                        return input
                    end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import converter-retry fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build converter-retry file"));
    let mut factory = RecordingFactory::new();
    let (mut session, creation) = flow_session::FlowSession::create_with_factory(
        file,
        flow_session::FlowSessionConfig::default(),
        &mut factory,
    )
    .expect("create converter-retry flow session");
    let creation_commands = creation
        .outputs
        .iter()
        .filter_map(|output| match &output.payload {
            flow_session::FlowOutputPayload::HostCommand { name, .. }
                if name.starts_with("converter_retry_") =>
            {
                Some(name.as_str())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        creation_commands,
        ["converter_retry_generated_1", "converter_retry_init_1",],
        "a failed user init neither marks didHydrate nor lets the failed lifetime convert"
    );

    let root = creation
        .bootstrap
        .catalog
        .root_instance_id
        .expect("fixture root instance");
    let rebind = session
        .perform_with_factory(
            flow_session::FlowOperation::StateBatch(flow_session::FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![flow_session::FlowStateMutation::SetValue {
                    instance: flow_session::FlowInstanceRef::Existing(root),
                    path: "amount".to_owned(),
                    value: flow_session::FlowScalarValue::Number(9.5),
                }],
                new_instances: Vec::new(),
            }),
            &mut factory,
        )
        .expect("retry converter on an explicit same-value rebind");
    let rebind_commands = rebind
        .outputs
        .iter()
        .filter_map(|output| match &output.payload {
            flow_session::FlowOutputPayload::HostCommand { name, .. }
                if name.starts_with("converter_retry_") =>
            {
                Some(name.as_str())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rebind_commands,
        [
            "converter_retry_generated_2",
            "converter_retry_init_2",
            "converter_retry_convert_2",
        ],
        "the next explicit C++ bind boundary recreates, hydrates, initializes, and wakes exactly the successful lifetime"
    );
}

#[test]
fn scripted_converter_valid_null_nested_input_preserves_field_and_continues_hydration() {
    let bytes = bound_listener_scripted_converter_file_with_missing_child(
        br#"
            return function(_context)
                return {
                    init = function(_self, _initContext) return true end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                nuxie.trigger("converter_atomic_generator")
                return {
                    initialized = false,
                    customChild = "sentinel",
                    init = function(self, _initContext)
                        if self.customAmount ~= 1.0 then
                            error("later authored inputs were not hydrated")
                        end
                        if self.customChild ~= "sentinel" then
                            error("a null selected child must leave the Lua field unchanged")
                        end
                        self.initialized = true
                        nuxie.trigger("converter_atomic_null_init")
                        return true
                    end,
                    convert = function(self, input)
                        if not self.initialized then
                            error("converter ran without a successful user init")
                        end
                        if self.customChild == "sentinel" then
                            nuxie.trigger("converter_atomic_null_convert")
                        elseif self.customChild ~= nil then
                            nuxie.trigger("converter_atomic_child_convert")
                        else
                            error("explicit rehydration did not install the selected child")
                        end
                        return input
                    end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import atomic-hydration fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build atomic-hydration file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate atomic-hydration artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate atomic-hydration machine");
    let mut factory = RecordingFactory::new();
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("bootstrap atomic-hydration scripts");
    let root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate authored root with unresolved child");
    assert!(
        root.handle()
            .linked_view_model_by_property_name_path("child")
            .is_none(),
        "the authored invalid child reference is the unresolved prerequisite"
    );
    let root_context = nuxie_runtime::RuntimeOwnedViewModelContextHandle::root(
        file.runtime(),
        root.handle().clone(),
    );
    machine.bind_owned_view_model_context_handle(&root_context);
    instantiate_script_listener_actions(&file, &mut machine, &mut factory, None)
        .expect("retain converter table through the machine-owned scoped context");
    instance.advance_with_state_machine(&mut machine, 0.0);
    let creation_commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name.starts_with("converter_atomic_") => {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        creation_commands,
        [
            "converter_atomic_generator",
            "converter_atomic_null_init",
            "converter_atomic_null_convert",
        ],
        "a valid ViewModel cell with a null selected child leaves that field unchanged, hydrates later inputs, runs init, and converts"
    );

    let child = ViewModelInstance {
        raw: RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(file.runtime(), 1)
                .expect("instantiate replacement child"),
        ),
    };
    assert!(
        root.handle()
            .link_view_model_by_property_name_path("child", child.handle())
            .expect("link compatible child")
    );
    let (action_global_id, input_global_id, converter_path) = machine
        .scripted_listener_data_converter_bind_steps()
        .into_iter()
        .find_map(|step| match step {
            nuxie_runtime::RuntimeScriptedListenerDataConverterBindStep::Rehydrate {
                action_global_id,
                listener_input_global_id,
                converter_path,
                ..
            } => Some((action_global_id, listener_input_global_id, converter_path)),
            _ => None,
        })
        .expect("fixture scripted converter rehydrate step");
    prepare_script_listener_data_converter_hydration(
        &file,
        &machine,
        action_global_id,
        input_global_id,
        &converter_path,
        None,
    )
    .expect("the machine-owned context resolves the complete converter preflight");
    let mut factory_option = Some(&mut factory as &mut dyn Factory);
    rehydrate_script_listener_actions(&file, &mut machine, None, None, &mut factory_option)
        .expect("rehydrate the retained converter table without a facade root fallback");
    instance.advance_with_state_machine(&mut machine, 0.0);
    let rebind_commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name.starts_with("converter_atomic_") => {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rebind_commands,
        ["converter_atomic_child_convert"],
        "explicit rehydration installs the selected child on the same initialized table"
    );
}

#[test]
fn scripted_converter_failed_init_regeneration_survives_a_later_invalid_preflight() {
    let bytes = bound_listener_scripted_converter_file_with_missing_child(
        br#"
            return function(_context)
                return {
                    init = function(_self, _initContext) return true end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        br#"
            local nuxie = require("nuxie")
            local generation = 0

            return function(_context)
                generation += 1
                local occurrence = generation
                nuxie.trigger("converter_preflight_generated_" .. occurrence)
                return {
                    init = function(self, _initContext)
                        nuxie.trigger("converter_preflight_init_" .. occurrence)
                        if occurrence == 1 then
                            return false
                        end
                        if self.customAmount ~= 1.0 or self.customChild == nil then
                            error("the regenerated table was initialized before full preflight")
                        end
                        return true
                    end,
                    convert = function(_self, input)
                        nuxie.trigger("converter_preflight_convert_" .. occurrence)
                        return input
                    end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import retry-preflight fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build retry-preflight file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate retry-preflight artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate retry-preflight machine");
    let mut factory = RecordingFactory::new();
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("bootstrap retry-preflight scripts");

    let valid_root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate first root context");
    let valid_child = ViewModelInstance {
        raw: RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(file.runtime(), 1).expect("instantiate first child"),
        ),
    };
    assert!(
        valid_root
            .handle()
            .link_view_model_by_property_name_path("child", valid_child.handle())
            .expect("attach first child")
    );
    let valid_context = nuxie_runtime::RuntimeOwnedViewModelContextHandle::root(
        file.runtime(),
        valid_root.handle().clone(),
    );
    machine.bind_owned_view_model_context_handle(&valid_context);
    instantiate_script_listener_actions(&file, &mut machine, &mut factory, None)
        .expect("first converter lifetime reaches its rejecting init");
    let first_commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name.starts_with("converter_preflight_") => {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        first_commands,
        [
            "converter_preflight_generated_1",
            "converter_preflight_init_1",
        ],
        "the first complete occurrence fails only at user init"
    );

    let unresolved_root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate unresolved replacement root");
    assert!(
        unresolved_root
            .handle()
            .linked_view_model_by_property_name_path("child")
            .is_none()
    );
    let unresolved_context = nuxie_runtime::RuntimeOwnedViewModelContextHandle::root(
        file.runtime(),
        unresolved_root.handle().clone(),
    );
    machine.bind_owned_view_model_context_handle(&unresolved_context);
    let mut factory_option = Some(&mut factory as &mut dyn Factory);
    rehydrate_script_listener_actions(&file, &mut machine, None, None, &mut factory_option)
        .expect("ordinary missing-input preflight remains inert");
    let unresolved_commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name.starts_with("converter_preflight_") => {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        unresolved_commands,
        [
            "converter_preflight_generated_2",
            "converter_preflight_init_2",
        ],
        "a resolved ViewModel property with a null child is a valid hydration prerequisite, so C++ leaves that table field unchanged and still calls init; this init failure disposes generation 2 (`script_input_viewmodel_property.cpp:60-113`; `scripted_object.cpp:277-303,399-437`)"
    );

    let replacement_child = ViewModelInstance {
        raw: RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(file.runtime(), 1)
                .expect("instantiate replacement child"),
        ),
    };
    assert!(
        unresolved_root
            .handle()
            .link_view_model_by_property_name_path("child", replacement_child.handle())
            .expect("attach replacement child")
    );
    machine.bind_owned_view_model_context_handle(&unresolved_context);
    rehydrate_script_listener_actions(&file, &mut machine, None, None, &mut factory_option)
        .expect("the regenerated table completes hydration and init");
    instance.advance_with_state_machine(&mut machine, 0.0);
    let recovered_commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name.starts_with("converter_preflight_") => {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        recovered_commands,
        [
            "converter_preflight_generated_3",
            "converter_preflight_init_3",
            "converter_preflight_convert_3",
        ],
        "generation 2 was disposed by its failing init, so the next explicit rebind recreates generation 3 before hydrating the now-live child (`scripted_object.cpp:277-303,313-437`)"
    );
}

#[test]
fn state_batch_commits_the_pre_callback_binding_source() {
    let bytes = bound_listener_input_file(
        br#"
            local nuxie = require("nuxie")

            return function(context)
                local performs = 0
                return {
                    init = function(_self, _initContext) return true end,
                    boundPulse = function(self)
                        if self.boundAmount ~= 17 then
                            error("batch scalar was not hydrated before its trigger")
                        end
                        context:viewModel().amount.value = 23
                        nuxie.trigger("binding_callback_wrote")
                    end,
                    performAction = function(self, _invocation)
                        performs += 1
                        if performs == 1 then
                            if self.boundAmount ~= 17 then
                                error("callback write did not remain pending")
                            end
                            if context:viewModel().amount.value ~= 23 then
                                error("listener Context could not read the live callback write")
                            end
                            nuxie.trigger("callback_write_pending")
                        elseif performs == 2 then
                            if self.boundAmount ~= 23 then
                                error("pending callback write was not flushed")
                            end
                            nuxie.trigger("callback_write_flushed")
                        end
                    end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import callback-write fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build callback-write file"));
    let mut factory = RecordingFactory::new();
    let (mut session, creation) = flow_session::FlowSession::create_with_factory(
        file,
        flow_session::FlowSessionConfig::default(),
        &mut factory,
    )
    .expect("create callback-write flow session");
    let root = creation
        .bootstrap
        .catalog
        .root_instance_id
        .expect("fixture root instance");

    let state_result = session
        .perform_with_factory(
            flow_session::FlowOperation::StateBatch(flow_session::FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![
                    flow_session::FlowStateMutation::SetValue {
                        instance: flow_session::FlowInstanceRef::Existing(root),
                        path: "amount".to_owned(),
                        value: flow_session::FlowScalarValue::Number(17.0),
                    },
                    flow_session::FlowStateMutation::FireTrigger {
                        instance: flow_session::FlowInstanceRef::Existing(root),
                        path: "pulse".to_owned(),
                    },
                ],
                new_instances: Vec::new(),
            }),
            &mut factory,
        )
        .expect("commit the batch and its binding callback");
    assert!(state_result.outputs.iter().any(|output| matches!(
        &output.payload,
        flow_session::FlowOutputPayload::HostCommand { name, .. }
            if name == "binding_callback_wrote"
    )));

    let pointer_result = session
        .perform_with_factory(
            flow_session::FlowOperation::PointerBatch(flow_session::FlowPointerBatch {
                events: vec![
                    flow_session::FlowPointerEvent {
                        kind: flow_session::FlowPointerKind::Down,
                        pointer_id: 1,
                        x: 0.0,
                        y: 0.0,
                        timestamp_seconds: 0.0,
                    },
                    flow_session::FlowPointerEvent {
                        kind: flow_session::FlowPointerKind::Down,
                        pointer_id: 2,
                        x: 0.0,
                        y: 0.0,
                        timestamp_seconds: 0.0,
                    },
                ],
            }),
            &mut factory,
        )
        .expect("flush the callback write between pointer subcycles");
    let commands = pointer_result
        .outputs
        .iter()
        .filter_map(|output| match &output.payload {
            flow_session::FlowOutputPayload::HostCommand { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        commands,
        ["callback_write_pending", "callback_write_flushed"]
    );
}

#[test]
fn pointer_listener_actions_precede_one_exact_binding_flush() {
    let bytes = pointer_trigger_binding_file(
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                return {
                    init = function(_self, _initContext) return true end,
                    boundPulse = function(_self)
                        nuxie.trigger("bound_trigger_fired")
                    end,
                    performAction = function(_self, _invocation)
                        nuxie.trigger("pointer_action")
                    end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import pointer-trigger fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build pointer-trigger file"));
    let mut factory = RecordingFactory::new();
    let (mut session, _) = flow_session::FlowSession::create_with_factory(
        file,
        flow_session::FlowSessionConfig::default(),
        &mut factory,
    )
    .expect("create pointer-trigger flow session");

    let pointer_result = session
        .perform_with_factory(
            flow_session::FlowOperation::PointerBatch(flow_session::FlowPointerBatch {
                events: vec![flow_session::FlowPointerEvent {
                    kind: flow_session::FlowPointerKind::Down,
                    pointer_id: 1,
                    x: 0.0,
                    y: 0.0,
                    timestamp_seconds: 0.0,
                }],
            }),
            &mut factory,
        )
        .expect("run pointer action and its retained binding flush");
    let pointer_commands = pointer_result
        .outputs
        .iter()
        .filter_map(|output| match &output.payload {
            flow_session::FlowOutputPayload::HostCommand { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        pointer_commands,
        ["pointer_action", "bound_trigger_fired"],
        "the binding edge flushes after the listener FIFO in the same cycle"
    );

    for timestamp_seconds in [1.0, 2.0] {
        let advance = session
            .perform_with_factory(
                flow_session::FlowOperation::Advance(flow_session::FlowAdvance {
                    timestamp_seconds,
                    delta_seconds: 0.0,
                    render: false,
                }),
                &mut factory,
            )
            .expect("advance after the consumed trigger edge");
        assert!(
            advance.outputs.iter().all(|output| !matches!(
                &output.payload,
                flow_session::FlowOutputPayload::HostCommand { name, .. }
                    if name == "bound_trigger_fired"
            )),
            "a retained trigger edge must be delivered exactly once"
        );
    }
}

#[test]
fn first_factory_pointer_prepares_and_applies_fixed_bindings_before_callback() {
    let bytes = bound_listener_input_file(
        br#"
            local nuxie = require("nuxie")

            return function(context)
                if context:viewModel() == nil then
                    error("pointer preparation did not install the session root")
                end
                return {
                    init = function(_self, _initContext) return true end,
                    performAction = function(self, _invocation)
                        if self.boundAmount ~= 9.5 then
                            error("pointer callback ran before updateDataBinds(false)")
                        end
                        nuxie.trigger("pointer_saw_bound_source")
                    end,
                }
            end
        "#,
    );
    let runtime =
        read_runtime_file_for_facade(&bytes).expect("import lazy pointer binding fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build lazy pointer binding file"));
    let (mut session, _) =
        flow_session::FlowSession::create(file, flow_session::FlowSessionConfig::default())
            .expect("create a cold session without renderer authority");
    let mut factory = RecordingFactory::new();

    let result = session
        .perform_with_factory(
            flow_session::FlowOperation::PointerBatch(flow_session::FlowPointerBatch {
                events: vec![flow_session::FlowPointerEvent {
                    kind: flow_session::FlowPointerKind::Down,
                    pointer_id: 1,
                    x: 50.0,
                    y: 50.0,
                    timestamp_seconds: 0.0,
                }],
            }),
            &mut factory,
        )
        .expect("prepare and dispatch the first factory-backed pointer");
    assert!(result.outputs.iter().any(|output| matches!(
        &output.payload,
        flow_session::FlowOutputPayload::HostCommand { name, .. }
            if name == "pointer_saw_bound_source"
    )));
}

#[test]
fn first_factory_advance_initializes_against_the_session_root() {
    let bytes = bound_listener_input_file(
        br#"
            local nuxie = require("nuxie")

            return function(context)
                if context:viewModel() == nil
                    or context:viewModel().amount.value ~= 9.5
                then
                    error("lazy advance generator did not receive the session root")
                end
                return {
                    init = function(self, initContext)
                        if initContext:viewModel() == nil
                            or self.boundChild == nil
                        then
                            error("lazy advance init missed its live DataContext")
                        end
                        nuxie.trigger("advance_initialized_with_root")
                        return true
                    end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
    );
    let runtime =
        read_runtime_file_for_facade(&bytes).expect("import lazy advance binding fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build lazy advance binding file"));
    let (mut session, _) =
        flow_session::FlowSession::create(file, flow_session::FlowSessionConfig::default())
            .expect("create a cold session without renderer authority");
    let mut factory = RecordingFactory::new();

    let result = session
        .perform_with_factory(
            flow_session::FlowOperation::Advance(flow_session::FlowAdvance {
                timestamp_seconds: 0.0,
                delta_seconds: 0.0,
                render: false,
            }),
            &mut factory,
        )
        .expect("run the first factory-backed advance");
    assert!(result.outputs.iter().any(|output| matches!(
        &output.payload,
        flow_session::FlowOutputPayload::HostCommand { name, .. }
            if name == "advance_initialized_with_root"
    )));
}

#[test]
fn move_and_exit_binding_edges_wait_for_the_next_run_cycle() {
    for (listener_type_value, events) in [
        (
            4,
            vec![flow_session::FlowPointerEvent {
                kind: flow_session::FlowPointerKind::Move,
                pointer_id: 1,
                x: 0.0,
                y: 0.0,
                timestamp_seconds: 0.0,
            }],
        ),
        (
            1,
            vec![
                flow_session::FlowPointerEvent {
                    kind: flow_session::FlowPointerKind::Move,
                    pointer_id: 1,
                    x: 0.0,
                    y: 0.0,
                    timestamp_seconds: 0.0,
                },
                flow_session::FlowPointerEvent {
                    kind: flow_session::FlowPointerKind::Exit,
                    pointer_id: 1,
                    x: 0.0,
                    y: 0.0,
                    timestamp_seconds: 0.0,
                },
            ],
        ),
    ] {
        let bytes = pointer_trigger_binding_file_for_listener(
            br#"
                local nuxie = require("nuxie")

                return function(_context)
                    return {
                        init = function(_self, _initContext) return true end,
                        boundPulse = function(_self)
                            nuxie.trigger("bound_trigger_fired")
                        end,
                        performAction = function(_self, _invocation)
                            nuxie.trigger("pointer_nonadvance")
                        end,
                    }
                end
            "#,
            listener_type_value,
        );
        let runtime = read_runtime_file_for_facade(&bytes)
            .expect("import non-advancing pointer-trigger fixture");
        let file = Arc::new(
            File::from_runtime(runtime).expect("build non-advancing pointer-trigger file"),
        );
        let mut factory = RecordingFactory::new();
        let (mut session, _) = flow_session::FlowSession::create_with_factory(
            file,
            flow_session::FlowSessionConfig::default(),
            &mut factory,
        )
        .expect("create non-advancing pointer-trigger flow session");

        let pointer_result = session
            .perform_with_factory(
                flow_session::FlowOperation::PointerBatch(flow_session::FlowPointerBatch {
                    events,
                }),
                &mut factory,
            )
            .expect("run the non-advancing pointer listener");
        let pointer_commands = pointer_result
            .outputs
            .iter()
            .filter_map(|output| match &output.payload {
                flow_session::FlowOutputPayload::HostCommand { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(pointer_commands, ["pointer_nonadvance"]);

        let advance = session
            .perform_with_factory(
                flow_session::FlowOperation::Advance(flow_session::FlowAdvance {
                    timestamp_seconds: 1.0,
                    delta_seconds: 0.0,
                    render: false,
                }),
                &mut factory,
            )
            .expect("flush the pending non-advancing pointer binding edge");
        let advance_commands = advance
            .outputs
            .iter()
            .filter_map(|output| match &output.payload {
                flow_session::FlowOutputPayload::HostCommand { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(advance_commands, ["bound_trigger_fired"]);
    }
}

#[test]
fn listener_init_failure_retries_during_constructor_initialization() {
    let (file, mut instance, mut machine, mut factory) = prepared_machine(
        br#"
            local generated = 0

            return function(_context)
                generated += 1
                local occurrence = generated
                return {
                    init = function(_self, _context)
                        return occurrence ~= 2
                    end,
                    performAction = function(_self, _invocation)
                    end,
                }
            end
        "#,
        2,
    );
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("bootstrap the file VM");
    let action_ids = machine
        .scripted_listener_actions()
        .iter()
        .map(|definition| definition.action_global_id())
        .collect::<Vec<_>>();

    instantiate_script_listener_actions(&file, &mut machine, &mut factory, None)
        .expect("ordinary init rejection is retained for a later retry");
    assert_eq!(action_ids.len(), 2);
    assert!(
        !machine
            .scripted_listener_action_user_init_pending(
                *action_ids.first().expect("first action id"),
            )
            .expect("first occurrence init state")
    );
    assert!(
        action_ids.iter().all(|action_id| !machine
            .scripted_listener_action_user_init_pending(*action_id)
            .expect("constructor retry occurrence init state")),
        "the post-context initScriptedObjects-equivalent immediately recreates the failed cold lifetime"
    );
}

#[test]
fn listener_cold_generator_cannot_see_an_already_owned_live_data_context() {
    let bytes = bound_listener_input_file(
        br#"
            local nuxie = require("nuxie")
            local generation = 0

            return function(context)
                generation += 1
                if context:viewModel() == nil then
                    -- A non-table result follows the same pinned failure path
                    -- as a generator error and forces the later live pass to
                    -- create a fresh occurrence.
                    return nil
                end
                return {
                    init = function(self, initContext)
                        if generation ~= 2 then
                            error("the cold generator did not run exactly once")
                        end
                        if initContext:viewModel() == nil
                            or self.boundChild == nil
                            or self.boundAmount ~= 1
                        then
                            error("the regenerated occurrence missed live hydration")
                        end
                        nuxie.trigger("listener_cold_then_live")
                        return true
                    end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import cold-generator fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build cold-generator file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate cold-generator artboard");
    let root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate cold-generator context");
    instance.bind_view_model(&root);
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate cold-generator machine");
    let mut factory = RecordingFactory::new();
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("bootstrap cold-generator scripts");

    instantiate_script_listener_actions(&file, &mut machine, &mut factory, None)
        .expect("run cold then live listener initialization");
    let commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name == "listener_cold_then_live" => Some(name),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        commands,
        ["listener_cold_then_live"],
        "clone/reinit sees nil despite the pre-bound machine context, then the live pass regenerates once (`scripted_listener_action.cpp:154-160`; `state_machine_instance.cpp:2072-2082`)"
    );
}

#[test]
fn listener_cold_table_waits_for_live_view_model_input_without_regeneration() {
    let bytes = bound_listener_input_file(
        br#"
            local nuxie = require("nuxie")
            local generation = 0

            return function(_context)
                generation += 1
                return {
                    init = function(self, initContext)
                        if generation ~= 1 then
                            error("the cold table was regenerated before live hydration")
                        end
                        if initContext:viewModel() == nil
                            or self.boundChild == nil
                            or self.boundAmount ~= 1
                        then
                            error("the retained cold table missed live hydration")
                        end
                        nuxie.trigger("listener_cold_table_live_init")
                        return true
                    end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import cold-table fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build cold-table file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate cold-table artboard");
    let root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate cold-table context");
    instance.bind_view_model(&root);
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate cold-table machine");
    let mut factory = RecordingFactory::new();
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("bootstrap cold-table scripts");

    instantiate_script_listener_actions(&file, &mut machine, &mut factory, None)
        .expect("retain cold table through live ViewModel hydration");
    let commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name == "listener_cold_table_live_init" => {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        commands,
        ["listener_cold_table_live_init"],
        "ViewModel ScriptInput cold validation retains one table, then the live pass hydrates and initializes it without another generator call (`script_input_viewmodel_property.cpp:46-81`; `scripted_object.cpp:399-437`)"
    );
}

#[test]
fn prebound_constructor_hydrates_deferred_listener_before_converter_binding() {
    let bytes = bound_listener_scripted_converter_file_with_listener_child(
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                return {
                    init = function(self, initContext)
                        if initContext:viewModel() == nil or self.boundChild == nil then
                            error("the constructor did not install the pre-bound DataContext")
                        end
                        nuxie.trigger("listener_preconverter_init")
                        return true
                    end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                return {
                    init = function(_self, _initContext)
                        nuxie.trigger("converter_bound")
                        return true
                    end,
                    convert = function(_self, _value) return 42 end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import constructor-order fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build constructor-order file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate constructor-order artboard");
    let root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate constructor-order context");
    instance.bind_view_model(&root);
    assert!(instance.owned_view_model_context().is_some());
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate constructor-order machine");
    let mut factory = RecordingFactory::new();
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("bootstrap constructor-order scripts");

    instantiate_script_listener_actions(&file, &mut machine, &mut factory, None)
        .expect("run the complete constructor lifecycle");
    let commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. }
                if matches!(
                    name.as_str(),
                    "listener_preconverter_init" | "converter_bound"
                ) =>
            {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        commands,
        ["listener_preconverter_init", "converter_bound"],
        "a pre-bound root constructor hydrates deferred fixed ScriptedObjects before `inheritDataContext` binds converter chains (`state_machine_instance.cpp:2072-2082`; `artboard.cpp:2844-2856`; `data_bind.cpp:251-328`)"
    );
}

#[test]
fn post_constructor_context_bind_runs_converter_before_live_listener_init() {
    let bytes = bound_listener_scripted_converter_file_with_listener_child(
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                return {
                    init = function(self, initContext)
                        if initContext:viewModel() == nil or self.boundChild == nil then
                            error("the explicit bind did not install its live DataContext")
                        end
                        nuxie.trigger("listener_postbind_init")
                        return true
                    end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                return {
                    init = function(_self, _initContext)
                        nuxie.trigger("converter_bound")
                        return true
                    end,
                    convert = function(_self, _value) return 42 end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import post-bind fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build post-bind file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate post-bind artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("construct machine without a DataContext");
    let mut factory = RecordingFactory::new();
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("bootstrap post-bind scripts");
    let root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate post-bind context");
    let root_context = nuxie_runtime::RuntimeOwnedViewModelContextHandle::root(
        file.runtime(),
        root.handle().clone(),
    );
    machine.bind_owned_view_model_context_handle(&root_context);

    instantiate_script_listener_actions(&file, &mut machine, &mut factory, None)
        .expect("run explicit-bind lifecycle");
    let commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. }
                if matches!(name.as_str(), "listener_postbind_init" | "converter_bound") =>
            {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        commands,
        ["converter_bound", "listener_postbind_init"],
        "a context attached after construction enters `internalDataContext`, whose DataBind/converter pass precedes `initScriptedObjects` (`state_machine_instance.cpp:2880-2913`; `data_bind.cpp:251-328`)"
    );
}

#[test]
fn listener_owned_empty_context_never_falls_back_to_the_facade_root() {
    let bytes = bound_listener_input_file(
        br#"
            local nuxie = require("nuxie")
            local generation = 0

            return function(_context)
                generation += 1
                nuxie.trigger("empty_listener_generator")
                return {
                    init = function(self, initContext)
                        if generation ~= 1 then
                            error("the unresolved table was regenerated")
                        end
                        if initContext:viewModel() == nil
                            or self.boundChild == nil
                            or self.boundAmount ~= 1
                        then
                            error("the owned live context did not hydrate")
                        end
                        nuxie.trigger("empty_listener_init")
                        return true
                    end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import empty-listener fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build empty-listener file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate empty-listener artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate empty-listener machine");
    let mut factory = RecordingFactory::new();
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("bootstrap empty-listener scripts");
    let root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate valid facade fallback");
    assert!(
        machine.bind_owned_view_model_contexts(&nuxie_runtime::RuntimeOwnedViewModelContext::new()),
        "install an occurrence-owned DataContext with no main instance"
    );

    instantiate_script_listener_actions(&file, &mut machine, &mut factory, Some(&root))
        .expect("empty owned context keeps the listener unresolved");
    let cold_commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name.starts_with("empty_listener_") => {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        cold_commands,
        ["empty_listener_generator"],
        "an unrelated facade root cannot initialize an occurrence whose own DataContext is empty"
    );

    let root_context = nuxie_runtime::RuntimeOwnedViewModelContextHandle::root(
        file.runtime(),
        root.handle().clone(),
    );
    machine.bind_owned_view_model_context_handle(&root_context);
    let mut factory_option = Some(&mut factory as &mut dyn Factory);
    rehydrate_script_listener_actions(&file, &mut machine, None, None, &mut factory_option)
        .expect("the same table hydrates after its owned context becomes live");
    let live_commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name.starts_with("empty_listener_") => {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        live_commands,
        ["empty_listener_init"],
        "the retained table initializes once from its occurrence-owned context (`lua_scripted_context.cpp:129-146`)"
    );
}

#[test]
fn converter_owned_empty_context_never_falls_back_to_the_facade_root() {
    let bytes = bound_listener_scripted_converter_file_with_missing_child(
        br#"
            return function(_context)
                return {
                    init = function(_self, _initContext) return true end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        br#"
            local nuxie = require("nuxie")
            local generation = 0

            return function(_context)
                generation += 1
                nuxie.trigger("empty_converter_generator")
                return {
                    init = function(self, _initContext)
                        if generation ~= 1 then
                            error("the unresolved converter was regenerated")
                        end
                        if self.customAmount ~= 1 or self.customChild == nil then
                            error("the owned live context did not hydrate converter inputs")
                        end
                        nuxie.trigger("empty_converter_init")
                        return true
                    end,
                    convert = function(_self, input)
                        nuxie.trigger("empty_converter_convert")
                        return input
                    end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import empty-converter fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build empty-converter file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate empty-converter artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate empty-converter machine");
    let mut factory = RecordingFactory::new();
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("bootstrap empty-converter scripts");
    let root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate valid facade fallback");
    let child = ViewModelInstance {
        raw: RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(file.runtime(), 1)
                .expect("instantiate compatible child"),
        ),
    };
    assert!(
        root.handle()
            .link_view_model_by_property_name_path("child", child.handle())
            .expect("attach valid fallback child")
    );
    assert!(
        machine.bind_owned_view_model_contexts(&nuxie_runtime::RuntimeOwnedViewModelContext::new()),
        "install an occurrence-owned DataContext with no main instance"
    );

    instantiate_script_listener_actions(&file, &mut machine, &mut factory, Some(&root))
        .expect("empty owned context keeps the converter unresolved");
    let cold_commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name.starts_with("empty_converter_") => {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        cold_commands,
        ["empty_converter_generator"],
        "converter custom inputs cannot leak through the valid facade fallback"
    );

    let root_context = nuxie_runtime::RuntimeOwnedViewModelContextHandle::root(
        file.runtime(),
        root.handle().clone(),
    );
    machine.bind_owned_view_model_context_handle(&root_context);
    let mut factory_option = Some(&mut factory as &mut dyn Factory);
    rehydrate_script_listener_actions(&file, &mut machine, None, None, &mut factory_option)
        .expect("the same converter table hydrates from its owned context");
    instance.advance_with_state_machine(&mut machine, 0.0);
    let live_commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name.starts_with("empty_converter_") => {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        live_commands,
        ["empty_converter_init", "empty_converter_convert"],
        "the retained converter initializes and converts once from its occurrence-owned context"
    );
}

#[test]
fn listener_missing_context_hydration_keeps_the_table_until_context_arrives() {
    let bytes = bound_listener_input_file(
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                return {
                    init = function(self, initContext)
                        if initContext:viewModel() == nil then
                            error("init ran before the data context arrived")
                        end
                        -- C++ hydrates the cloned authored scalar before the
                        -- later updateDataBinds(false) source application.
                        if self.boundChild == nil or self.boundAmount ~= 1 then
                            error("deferred inputs did not pass one write-free preflight")
                        end
                        nuxie.trigger("deferred_listener_initialized")
                        return true
                    end,
                    performAction = function(_self, _invocation)
                        nuxie.trigger("pending_listener_performed")
                    end,
                }
            end
        "#,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import failed-hydration fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build failed-hydration file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate failed-hydration artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate failed-hydration machine");
    let mut factory = RecordingFactory::new();
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("bootstrap failed-hydration VM");
    let action_id = machine
        .scripted_listener_actions()
        .first()
        .expect("fixture listener action")
        .action_global_id();

    instantiate_script_listener_actions(&file, &mut machine, &mut factory, None)
        .expect("a missing context defers hydration without failing the owner");
    assert!(
        machine.has_scripted_listener_action_instance(action_id),
        "the cold generator table remains owned by this occurrence"
    );
    assert!(
        machine
            .scripted_listener_action_user_init_pending(action_id)
            .expect("deferred init state")
    );
    assert!(
        machine.pointer_down(&mut instance.raw, 50.0, 50.0, 1),
        "the pending listener still owns its C++ m_self table"
    );
    assert!(instance.drain_flow_host_commands().iter().any(|command| {
        matches!(
            command,
            LuaHostCommand::Trigger { name, .. }
                if name == "pending_listener_performed"
        )
    }));

    let root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate the authored root context");
    let mut factory_option = Some(&mut factory as &mut dyn Factory);
    rehydrate_script_listener_actions(&file, &mut machine, Some(&root), None, &mut factory_option)
        .expect("the live context completes deferred hydration and init");
    assert!(
        !machine
            .scripted_listener_action_user_init_pending(action_id)
            .expect("completed deferred init state")
    );
    assert!(instance.drain_flow_host_commands().iter().any(|command| {
        matches!(
            command,
            LuaHostCommand::Trigger { name, .. }
                if name == "deferred_listener_initialized"
        )
    }));
}

#[test]
fn listener_generator_and_init_do_not_require_a_new_renderer_factory() {
    let bytes = scripted_listener_file(
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                nuxie.trigger("factoryless_listener_generated")
                return {
                    init = function(_self, _initContext)
                        nuxie.trigger("factoryless_listener_initialized")
                        return true
                    end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        1,
    );
    let runtime =
        read_runtime_file_for_facade(&bytes).expect("import factoryless listener fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build factoryless listener file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate factoryless listener artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate factoryless listener machine");
    let mut factory = RecordingFactory::new();
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("register the protocol while the renderer factory is available");

    let mut no_factory = None;
    rehydrate_script_listener_actions(&file, &mut machine, None, None, &mut no_factory)
        .expect("generator/init are scripting-VM operations");

    let commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name.starts_with("factoryless_listener_") => {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        commands,
        [
            "factoryless_listener_generated",
            "factoryless_listener_initialized",
        ],
        "pinned ScriptedObject generation and user init do not depend on a renderer Factory"
    );
}

#[test]
fn listener_failed_init_recreates_without_a_new_renderer_factory() {
    let bytes = scripted_listener_file(
        br#"
            local nuxie = require("nuxie")
            local generation = 0

            return function(_context)
                generation += 1
                local occurrence = generation
                nuxie.trigger("factoryless_retry_generated_" .. occurrence)
                return {
                    init = function(_self, _initContext)
                        nuxie.trigger("factoryless_retry_init_" .. occurrence)
                        return occurrence > 1
                    end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        1,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import factoryless retry fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build factoryless retry file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate factoryless retry artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate factoryless retry machine");
    let mut factory = RecordingFactory::new();
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("register retry protocol");

    let mut no_factory = None;
    rehydrate_script_listener_actions(&file, &mut machine, None, None, &mut no_factory)
        .expect("first factoryless init rejection is retained");
    rehydrate_script_listener_actions(&file, &mut machine, None, None, &mut no_factory)
        .expect("factoryless retry recreates and initializes");

    let commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name.starts_with("factoryless_retry_") => {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        commands,
        [
            "factoryless_retry_generated_1",
            "factoryless_retry_init_1",
            "factoryless_retry_generated_2",
            "factoryless_retry_init_2",
        ],
        "the failed m_self lifetime is recreated at the next scripting boundary without requiring a renderer Factory"
    );
}

#[test]
fn converter_generation_init_and_retry_do_not_require_a_new_renderer_factory() {
    let bytes = bound_listener_scripted_converter_file(
        br#"
            return function(_context)
                return {
                    init = function(_self, _initContext) return true end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        br#"
            local nuxie = require("nuxie")
            local generation = 0

            return function(context)
                if context:viewModel() == nil then
                    error("converter requires its live occurrence context")
                end
                generation += 1
                local occurrence = generation
                nuxie.trigger("factoryless_converter_generated_" .. occurrence)
                return {
                    init = function(_self, _initContext)
                        nuxie.trigger("factoryless_converter_init_" .. occurrence)
                        return occurrence > 1
                    end,
                    convert = function(_self, input) return input end,
                }
            end
        "#,
    );
    let runtime =
        read_runtime_file_for_facade(&bytes).expect("import factoryless converter fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build factoryless converter file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate factoryless converter artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate factoryless converter machine");
    let mut factory = RecordingFactory::new();
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("register converter protocol while the renderer factory is available");
    let root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate converter root context");
    let root_context = nuxie_runtime::RuntimeOwnedViewModelContextHandle::root(
        file.runtime(),
        root.handle().clone(),
    );
    machine.bind_owned_view_model_context_handle(&root_context);

    let mut no_factory = None;
    rehydrate_script_listener_actions(&file, &mut machine, Some(&root), None, &mut no_factory)
        .expect("first factoryless converter init rejection is retained");
    rehydrate_script_listener_actions(
        &file,
        &mut machine,
        Some(&root),
        Some(&root),
        &mut no_factory,
    )
    .expect("factoryless converter retry recreates and initializes");

    let commands = instance
        .drain_flow_host_commands()
        .into_iter()
        .filter_map(|command| match command {
            LuaHostCommand::Trigger { name, .. } if name.starts_with("factoryless_converter_") => {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        commands,
        [
            "factoryless_converter_generated_1",
            "factoryless_converter_init_1",
            "factoryless_converter_generated_2",
            "factoryless_converter_init_2",
        ],
        "pinned ScriptAsset generation and ScriptedObject retry are VM operations even when no renderer Factory is currently installed"
    );
}

#[test]
fn public_update_data_binds_reconciles_a_machine_with_only_a_cloned_script_input_bind() {
    const DATA_BIND_TO_SOURCE: u64 = 1 << 0;
    let bytes = bound_listener_input_file_with_amount_flags(
        br#"
            return function(_context)
                return {
                    init = function(_self, _context) return true end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        DATA_BIND_TO_SOURCE,
    );
    let runtime = read_runtime_file_for_facade(&bytes).expect("import public-update fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build public-update file"));
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate public-update artboard");
    let mut machine = instance
        .default_state_machine_instance()
        .expect("instantiate public-update machine");
    let mut factory = RecordingFactory::new();
    instance
        .prepare_flow_scripts(&mut factory)
        .expect("bootstrap public-update VM");
    let root = instance
        .instantiate_view_model_instance(0)
        .expect("instantiate public-update context");
    assert_eq!(
        root.raw().number_value_by_property_name("amount"),
        Some(9.5)
    );
    assert!(
        machine.begin_scripted_object_data_context_bind(root.handle()),
        "stage the public root as an actual DataContext before constructor completion"
    );
    instantiate_script_listener_actions(&file, &mut machine, &mut factory, Some(&root))
        .expect("mount cloned ScriptInput binding");
    assert_eq!(
        root.raw().number_value_by_property_name("amount"),
        Some(9.5),
        "ordinary updateDataBinds(false) must not pull the cloned target"
    );

    assert!(
        machine.update_data_binds_apply_target_to_source(),
        "the cloned ScriptInput container is a live public-update owner even when the ordinary graph has no binds"
    );
    assert_eq!(
        root.raw().number_value_by_property_name("amount"),
        Some(1.0),
        "public updateDataBinds(true) pulls the cloned authored target into the retained source"
    );
}

#[test]
fn failed_module_registration_attempt_rolls_back_only_its_host_effects() {
    let bytes = module_retry_listener_file();
    let runtime = read_runtime_file_for_facade(&bytes).expect("import module-retry fixture");
    let file = Arc::new(File::from_runtime(runtime).expect("build module-retry file"));
    let mut factory = RecordingFactory::new();
    let (_session, creation) = flow_session::FlowSession::create_with_factory(
        file,
        flow_session::FlowSessionConfig::default(),
        &mut factory,
    )
    .expect("module dependency retry converges");
    let commands = creation
        .outputs
        .iter()
        .filter(|output| {
            matches!(
                &output.payload,
                flow_session::FlowOutputPayload::HostCommand { name, .. }
                    if name == "module_a_registered"
            )
        })
        .count();
    assert_eq!(
        commands, 1,
        "the failed A attempt must not leak its pre-require host effect"
    );
}
