use nuxie_binary::{RuntimeFile, RuntimeObject, read_runtime_file};
use nuxie_runtime::{RuntimeOwnedViewModelInstance, ScriptValue, bound_script_input_value};
use nuxie_schema::definition_by_name;

const DATA_BIND_TO_SOURCE: u64 = 1 << 0;

#[derive(Clone, Copy)]
enum SourceKind {
    Boolean,
    Number,
    Trigger,
}

#[derive(Clone, Copy)]
enum ConverterFixture {
    None,
    Unresolved,
    ToNumber,
    NumberOperationGroup,
    Scripted,
    Interpolator,
}

impl ConverterFixture {
    fn converter_id(self) -> Option<u64> {
        match self {
            Self::None => None,
            Self::Unresolved => Some(99),
            Self::ToNumber => Some(0),
            Self::NumberOperationGroup => Some(2),
            Self::Scripted | Self::Interpolator => Some(0),
        }
    }
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

fn type_key(type_name: &str) -> u16 {
    definition_by_name(type_name)
        .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
        .type_key
        .int
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = definition_by_name(type_name)
        .unwrap_or_else(|| panic!("missing schema definition {type_name}"));
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            definition_by_name(ancestor)
                .unwrap_or_else(|| panic!("missing ancestor schema definition {ancestor}"))
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .unwrap_or_else(|| panic!("missing property {type_name}.{property_name}"))
        .key
        .int
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(bytes, u64::from(type_key(type_name)));
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

fn push_bool(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: bool) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    bytes.push(u8::from(value));
}

fn push_blob(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &[u8]) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value);
}

fn push_string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
    push_blob(bytes, type_name, name, value.as_bytes());
}

fn script_input_file(
    file_id: u64,
    source_kind: SourceKind,
    converter: ConverterFixture,
    data_bind_flags: u64,
) -> Vec<u8> {
    script_input_file_with_target(
        file_id,
        source_kind,
        converter,
        data_bind_flags,
        "ScriptInputNumber",
    )
}

fn script_input_file_with_target(
    file_id: u64,
    source_kind: SourceKind,
    converter: ConverterFixture,
    data_bind_flags: u64,
    target_type: &str,
) -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, file_id);
    push_var_uint(&mut bytes, 0);

    push_object(&mut bytes, "ViewModel", |bytes| {
        push_string(bytes, "ViewModel", "name", "Root");
    });
    match source_kind {
        SourceKind::Boolean => push_object(&mut bytes, "ViewModelPropertyBoolean", |bytes| {
            push_string(bytes, "ViewModelPropertyBoolean", "name", "source");
        }),
        SourceKind::Number => push_object(&mut bytes, "ViewModelPropertyNumber", |bytes| {
            push_string(bytes, "ViewModelPropertyNumber", "name", "source");
        }),
        SourceKind::Trigger => push_object(&mut bytes, "ViewModelPropertyTrigger", |bytes| {
            push_string(bytes, "ViewModelPropertyTrigger", "name", "source");
        }),
    }
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ViewModelInstance", |bytes| {
        push_string(bytes, "ViewModelInstance", "name", "root-default");
        push_uint(bytes, "ViewModelInstance", "viewModelId", 0);
    });
    match source_kind {
        SourceKind::Boolean => push_object(&mut bytes, "ViewModelInstanceBoolean", |bytes| {
            push_uint(bytes, "ViewModelInstanceBoolean", "viewModelPropertyId", 0);
            push_bool(bytes, "ViewModelInstanceBoolean", "propertyValue", true);
        }),
        SourceKind::Number => push_object(&mut bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_f32(bytes, "ViewModelInstanceNumber", "propertyValue", 0.0);
        }),
        SourceKind::Trigger => push_object(&mut bytes, "ViewModelInstanceTrigger", |bytes| {
            push_uint(bytes, "ViewModelInstanceTrigger", "viewModelPropertyId", 0);
            push_uint(bytes, "ViewModelInstanceTrigger", "propertyValue", 0);
        }),
    }
    match converter {
        ConverterFixture::None | ConverterFixture::Unresolved => {}
        ConverterFixture::ToNumber => push_object(&mut bytes, "DataConverterToNumber", |_| {}),
        ConverterFixture::NumberOperationGroup => {
            push_object(&mut bytes, "DataConverterOperationValue", |bytes| {
                push_uint(bytes, "DataConverterOperationValue", "operationType", 2);
                push_f32(bytes, "DataConverterOperationValue", "operationValue", 2.0);
            });
            push_object(&mut bytes, "DataConverterRounder", |bytes| {
                push_uint(bytes, "DataConverterRounder", "decimals", 1);
            });
            push_object(&mut bytes, "DataConverterGroup", |_| {});
            push_object(&mut bytes, "DataConverterGroupItem", |bytes| {
                push_uint(bytes, "DataConverterGroupItem", "converterId", 0);
            });
            push_object(&mut bytes, "DataConverterGroupItem", |bytes| {
                push_uint(bytes, "DataConverterGroupItem", "converterId", 1);
            });
        }
        ConverterFixture::Scripted => {
            push_object(&mut bytes, "ScriptedDataConverter", |_| {});
        }
        ConverterFixture::Interpolator => {
            push_object(&mut bytes, "DataConverterInterpolator", |bytes| {
                push_f32(bytes, "DataConverterInterpolator", "duration", 1.0);
            });
        }
    }
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "BindingProbe");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &[0]);
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
    push_object(&mut bytes, "StateMachine", |_| {});
    push_object(&mut bytes, "StateMachineListenerSingle", |bytes| {
        push_uint(bytes, "StateMachineListener", "targetId", 1);
        push_uint(bytes, "StateMachineListenerSingle", "listenerTypeValue", 2);
    });
    push_object(&mut bytes, "ScriptedListenerAction", |bytes| {
        push_uint(bytes, "ScriptedListenerAction", "scriptAssetId", 0);
    });
    push_object(&mut bytes, target_type, |bytes| {
        push_string(bytes, target_type, "name", "boundValue");
        if target_type == "ScriptInputNumber" {
            push_f32(bytes, target_type, "propertyValue", 7.0);
        }
    });
    let mut source_path = Vec::new();
    push_var_uint(&mut source_path, 0);
    push_var_uint(&mut source_path, 0);
    push_object(&mut bytes, "DataBindContext", |bytes| {
        push_uint(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key(target_type, "propertyValue")),
        );
        push_blob(bytes, "DataBindContext", "sourcePathIds", &source_path);
        if let Some(converter_id) = converter.converter_id() {
            push_uint(bytes, "DataBindContext", "converterId", converter_id);
        }
        if data_bind_flags != 0 {
            push_uint(bytes, "DataBindContext", "flags", data_bind_flags);
        }
    });
    bytes
}

fn import_fixture(bytes: &[u8]) -> RuntimeFile {
    read_runtime_file(bytes).expect("import synthetic ScriptInput binding fixture")
}

fn number_input(runtime: &RuntimeFile) -> &RuntimeObject {
    (0..runtime.object_count())
        .filter_map(|id| runtime.object(id))
        .find(|object| object.type_name == "ScriptInputNumber")
        .expect("fixture contains an imported ScriptInputNumber")
}

fn trigger_input(runtime: &RuntimeFile) -> &RuntimeObject {
    (0..runtime.object_count())
        .filter_map(|id| runtime.object(id))
        .find(|object| object.type_name == "ScriptInputTrigger")
        .expect("fixture contains an imported ScriptInputTrigger")
}

#[test]
fn trigger_binding_is_not_hydrated_as_an_initial_scalar_value() {
    let runtime = import_fixture(&script_input_file_with_target(
        9_507,
        SourceKind::Trigger,
        ConverterFixture::None,
        0,
        "ScriptInputTrigger",
    ));
    let context = RuntimeOwnedViewModelInstance::new(&runtime, 0)
        .expect("fixture exposes the Root view model");

    let value = bound_script_input_value(&runtime, &context, trigger_input(&runtime))
        .expect("initial scalar hydration must ignore trigger inputs");
    assert_eq!(
        value, None,
        "initialization must neither fire nor scalar-hydrate a trigger input"
    );
}

#[test]
fn omitted_converter_uses_the_authored_pass_through_binding() {
    let runtime = import_fixture(&script_input_file(
        9_500,
        SourceKind::Number,
        ConverterFixture::None,
        0,
    ));
    let mut context = RuntimeOwnedViewModelInstance::new(&runtime, 0)
        .expect("fixture exposes the Root view model");
    assert!(context.set_number_by_property_name("source", 12.5));

    let value = bound_script_input_value(&runtime, &context, number_input(&runtime))
        .expect("the missing-id converter sentinel means no converter");
    assert_eq!(value, Some(ScriptValue::Number(12.5)));
}

#[test]
fn boolean_to_number_converter_hydrates_a_number_script_input() {
    let runtime = import_fixture(&script_input_file(
        9_501,
        SourceKind::Boolean,
        ConverterFixture::ToNumber,
        0,
    ));
    let mut context = RuntimeOwnedViewModelInstance::new(&runtime, 0)
        .expect("fixture exposes the Root view model");
    assert!(context.set_boolean_by_property_name("source", true));

    let value = bound_script_input_value(&runtime, &context, number_input(&runtime))
        .expect("ToNumber is in the supported stateless converter subset");
    assert_eq!(value, Some(ScriptValue::Number(1.0)));
}

#[test]
fn stateless_number_converter_group_hydrates_in_authored_order() {
    let runtime = import_fixture(&script_input_file(
        9_502,
        SourceKind::Number,
        ConverterFixture::NumberOperationGroup,
        0,
    ));
    let mut context = RuntimeOwnedViewModelInstance::new(&runtime, 0)
        .expect("fixture exposes the Root view model");
    assert!(context.set_number_by_property_name("source", 0.44));

    let value = bound_script_input_value(&runtime, &context, number_input(&runtime))
        .expect("the operation-plus-rounder group is stateless");
    let Some(ScriptValue::Number(value)) = value else {
        panic!("converter group did not produce a number");
    };
    assert!((value - 0.9).abs() < 1e-6, "converted value was {value}");
}

#[test]
fn target_to_source_only_binding_leaves_the_script_input_unbound() {
    let runtime = import_fixture(&script_input_file(
        9_503,
        SourceKind::Number,
        ConverterFixture::None,
        DATA_BIND_TO_SOURCE,
    ));
    let mut context = RuntimeOwnedViewModelInstance::new(&runtime, 0)
        .expect("fixture exposes the Root view model");
    assert!(context.set_number_by_property_name("source", 42.0));

    let value = bound_script_input_value(&runtime, &context, number_input(&runtime))
        .expect("direction filtering is not an evaluation failure");
    assert_eq!(
        value, None,
        "a target-to-source-only binding must preserve the authored ScriptInput value"
    );
}

#[test]
fn explicit_unresolved_converter_fails_closed() {
    let runtime = import_fixture(&script_input_file(
        9_506,
        SourceKind::Number,
        ConverterFixture::Unresolved,
        0,
    ));
    let mut context = RuntimeOwnedViewModelInstance::new(&runtime, 0)
        .expect("fixture exposes the Root view model");
    assert!(context.set_number_by_property_name("source", 3.0));

    let error = bound_script_input_value(&runtime, &context, number_input(&runtime))
        .expect_err("an explicit broken converter reference must never behave as passthrough");
    assert!(
        error.message().contains("unresolved data converter"),
        "unexpected unresolved converter error: {error}"
    );
}

#[test]
fn scripted_data_converter_fails_closed_instead_of_passing_the_source_through() {
    let runtime = import_fixture(&script_input_file(
        9_504,
        SourceKind::Number,
        ConverterFixture::Scripted,
        0,
    ));
    let mut context = RuntimeOwnedViewModelInstance::new(&runtime, 0)
        .expect("fixture exposes the Root view model");
    assert!(context.set_number_by_property_name("source", 3.0));

    let error = bound_script_input_value(&runtime, &context, number_input(&runtime))
        .expect_err("an unattached ScriptedDataConverter must never behave as passthrough");
    assert!(
        error.message().contains("ScriptedDataConverter")
            && error
                .message()
                .contains("requires retained converter state"),
        "unexpected scripted converter error: {error}"
    );
}

#[test]
fn stateful_interpolator_fails_closed_without_occurrence_state() {
    let runtime = import_fixture(&script_input_file(
        9_505,
        SourceKind::Number,
        ConverterFixture::Interpolator,
        0,
    ));
    let mut context = RuntimeOwnedViewModelInstance::new(&runtime, 0)
        .expect("fixture exposes the Root view model");
    assert!(context.set_number_by_property_name("source", 3.0));

    let error = bound_script_input_value(&runtime, &context, number_input(&runtime))
        .expect_err("an interpolator requires retained occurrence-local state");
    assert!(
        error.message().contains("DataConverterInterpolator")
            && error
                .message()
                .contains("requires retained converter state"),
        "unexpected interpolator error: {error}"
    );
}
