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
    assert!(!output.is_null());
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
    let definition = definition_by_name(type_name).unwrap();
    definition
        .properties
        .iter()
        .chain(
            definition
                .ancestors
                .iter()
                .flat_map(|ancestor| definition_by_name(ancestor).unwrap().properties.iter()),
        )
        .find(|property| property.name == property_name)
        .unwrap()
        .key
        .int
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(definition_by_name(type_name).unwrap().type_key.int),
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

pub fn scripted_view_model_asset_fixture(source: &[u8]) -> Vec<u8> {
    let mut payload = vec![0];
    payload.extend(compile_luau(source));

    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 18_253, 0] {
        push_var_uint(&mut bytes, value);
    }
    push_object(&mut bytes, "ViewModel", |bytes| {
        push_string(bytes, "ViewModel", "name", "Root")
    });
    push_object(&mut bytes, "ViewModelPropertyNumber", |bytes| {
        push_string(bytes, "ViewModelPropertyNumber", "name", "amount")
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
        push_string(bytes, "ViewModelPropertyTrigger", "name", "pulse")
    });
    push_object(&mut bytes, "ViewModel", |bytes| {
        push_string(bytes, "ViewModel", "name", "Child")
    });
    push_object(&mut bytes, "ViewModelPropertyNumber", |bytes| {
        push_string(bytes, "ViewModelPropertyNumber", "name", "score")
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
        push_f32(bytes, "ViewModelInstanceNumber", "propertyValue", 0.0);
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
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "GenericHostChanges");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &payload);
    });
    push_object(&mut bytes, "ImageAsset", |bytes| {
        push_uint(bytes, "ImageAsset", "assetId", 7);
        push_string(bytes, "ImageAsset", "name", "pixel.png");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(
            bytes,
            "FileAssetContents",
            "bytes",
            include_bytes!(
                "../../../../tests/ExperienceRuntimeHostApp/Fixtures/external-image/assets/sha256/b9d4e51e3590796b9a65fc9ec0b623bdf71a2bacef0098b79063edc87055b1a0.png"
            ),
        );
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
        push_f32(bytes, "Artboard", "height", 100.0);
        push_uint(bytes, "Artboard", "viewModelId", 0);
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
        push_f32(bytes, "ParametricPath", "width", 100.0);
        push_f32(bytes, "ParametricPath", "height", 100.0);
    });
    push_object(&mut bytes, "StateMachine", |bytes| {
        push_string(bytes, "StateMachine", "name", "HostCommands");
    });
    push_object(&mut bytes, "StateMachineListenerSingle", |bytes| {
        push_uint(bytes, "StateMachineListener", "targetId", 1);
        push_uint(bytes, "StateMachineListenerSingle", "listenerTypeValue", 2);
    });
    push_object(&mut bytes, "ScriptedListenerAction", |bytes| {
        push_uint(bytes, "ScriptedListenerAction", "scriptAssetId", 0);
    });
    let mut amount_path = Vec::new();
    push_var_uint(&mut amount_path, 0);
    push_var_uint(&mut amount_path, 0);
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
