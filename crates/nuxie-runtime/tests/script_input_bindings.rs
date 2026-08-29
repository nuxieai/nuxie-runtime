use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        state_machine::StateMachine, state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    generated::core_registry::CoreRegistry,
    scripted::scripted_object::ScriptedObject,
};
use nuxie_runtime::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
    ScriptViewModel,
};
use nuxie_schema::definition_by_name;

const DATA_BIND_TO_SOURCE: u64 = 1 << 0;
const DATA_BIND_TWO_WAY: u64 = 1 << 1;

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
    MissingItemGroup,
    Scripted,
    Interpolator,
    PlainDataBindShadow,
}

impl ConverterFixture {
    fn converter_id(self) -> Option<u64> {
        match self {
            Self::None => None,
            Self::Unresolved => Some(99),
            Self::ToNumber => Some(0),
            Self::NumberOperationGroup | Self::MissingItemGroup => Some(2),
            Self::Scripted | Self::Interpolator => Some(0),
            Self::PlainDataBindShadow => None,
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
        ConverterFixture::None
        | ConverterFixture::Unresolved
        | ConverterFixture::PlainDataBindShadow => {}
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
        ConverterFixture::MissingItemGroup => {
            push_object(&mut bytes, "DataConverterOperationValue", |bytes| {
                push_uint(bytes, "DataConverterOperationValue", "operationType", 2);
                push_f32(bytes, "DataConverterOperationValue", "operationValue", 2.0);
            });
            push_object(&mut bytes, "DataConverterRounder", |bytes| {
                push_uint(bytes, "DataConverterRounder", "decimals", 1);
            });
            push_object(&mut bytes, "DataConverterGroup", |_| {});
            push_object(&mut bytes, "DataConverterGroupItem", |bytes| {
                push_uint(bytes, "DataConverterGroupItem", "converterId", 99);
            });
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
    if matches!(converter, ConverterFixture::PlainDataBindShadow) {
        push_object(&mut bytes, "DataBind", |_| {});
    }
    bytes
}

// Retain the actual imported scene. Input selection is the cloned ScriptInput
// belonging to this StateMachineInstance, never a descriptor-side evaluator.
struct Fixture {
    _file: RuntimeFileHandle,
    _artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    input: CoreHandle,
    model: ScriptViewModel,
}
impl Fixture {
    fn import(bytes: &[u8]) -> Self {
        let mut factory = PersistentFactory::new(RecordingFactory::default());
        let file = File::import(
            bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .expect("import synthetic ScriptInput binding fixture");
        let artboard = file.with_file(File::artboard_default).unwrap();
        let machine = artboard.state_machine_instance_handle(0).unwrap();
        let definition = machine.with_instance(|machine| machine.state_machine());
        let source = definition
            .with_downcast::<StateMachine, _>(|machine| {
                machine.scripted_objects().into_iter().next()
            })
            .flatten()
            .expect("authored scripted listener");
        let occurrence = machine
            .with_instance(|machine| machine.scripted_object(&source))
            .unwrap();
        let properties = ScriptedObject::custom_properties(&occurrence);
        assert_eq!(properties.len(), 1);
        let definition = file.with_file(|file| file.view_model(0)).unwrap();
        let model = nuxie_runtime::source::viewmodel::viewmodel::ViewModel::create_instance_handle(
            &definition,
        )
        .unwrap();
        let model = ScriptViewModel::from_native(model, file.clone()).unwrap();
        Self {
            _file: file,
            _artboard: artboard,
            machine,
            input: properties[0].clone(),
            model,
        }
    }

    fn bind(&self) {
        self.machine.with_instance_mut(|machine| {
            machine.bind_view_model_instance(self.model.native_instance().unwrap())
        });
        self.machine.advance_and_apply(0.0);
    }

    fn number(&self) -> f32 {
        CoreRegistry::get_double_handle(
            &self.input,
            i32::from(property_key("ScriptInputNumber", "propertyValue")),
        )
        .expect("actual ScriptInputNumber value")
    }

    fn bind_handle(&self) -> CoreHandle {
        self.input
            .with(|input| input.script_input_data_bind())
            .flatten()
            .unwrap()
    }
}

fn number_fixture(id: u64, converter: ConverterFixture, flags: u64) -> Fixture {
    Fixture::import(&script_input_file(id, SourceKind::Number, converter, flags))
}

#[test]
fn trigger_binding_is_not_hydrated_as_an_initial_scalar_value() {
    let fixture = Fixture::import(&script_input_file_with_target(
        9_507,
        SourceKind::Trigger,
        ConverterFixture::None,
        0,
        "ScriptInputTrigger",
    ));
    fixture.bind();
    let value = CoreRegistry::get_uint_handle(
        &fixture.input,
        i32::from(property_key("ScriptInputTrigger", "propertyValue")),
    )
    .unwrap();
    assert_eq!(
        value, 0,
        "initialization must neither fire nor scalar-hydrate a trigger input"
    );
}

#[test]
fn omitted_converter_uses_the_authored_pass_through_binding() {
    let fixture = number_fixture(9_500, ConverterFixture::None, 0);
    assert!(fixture.model.set_number("source", 12.5));
    fixture.bind();
    assert_eq!(fixture.number(), 12.5);
}

#[test]
fn boolean_to_number_converter_hydrates_a_number_script_input() {
    let fixture = Fixture::import(&script_input_file(
        9_501,
        SourceKind::Boolean,
        ConverterFixture::ToNumber,
        0,
    ));
    assert!(fixture.model.set_boolean("source", true));
    fixture.bind();
    assert_eq!(fixture.number(), 1.0);
}

#[test]
fn stateless_number_converter_group_hydrates_in_authored_order() {
    let fixture = number_fixture(9_502, ConverterFixture::NumberOperationGroup, 0);
    assert!(fixture.model.set_number("source", 0.44));
    fixture.bind();
    let value = fixture.number();
    assert!((value - 0.9).abs() < 1e-6, "converted value was {value}");
}

#[test]
fn converter_group_skips_null_items_and_keeps_authored_order() {
    let fixture = number_fixture(9_509, ConverterFixture::MissingItemGroup, 0);
    assert!(fixture.model.set_number("source", 0.44));
    fixture.bind();
    let value = fixture.number();
    assert!((value - 0.9).abs() < 1e-6, "converted value was {value}");
}

#[test]
fn later_plain_data_bind_shadows_the_earlier_context_bind() {
    let fixture = number_fixture(9_510, ConverterFixture::PlainDataBindShadow, 0);
    assert!(fixture.model.set_number("source", 42.0));
    let bind = fixture.bind_handle();
    assert_eq!(
        bind.core_type(),
        Some(type_key("DataBind")),
        "ScriptInput retains only the last authored DataBind subclass"
    );
    fixture.bind();
    assert_eq!(
        fixture.number(),
        7.0,
        "shadowed context binding leaves the authored input intact"
    );
}

#[test]
fn target_to_source_only_binding_leaves_the_script_input_unbound() {
    let fixture = number_fixture(9_503, ConverterFixture::None, DATA_BIND_TO_SOURCE);
    assert!(fixture.model.set_number("source", 42.0));
    fixture.bind();
    assert_eq!(
        fixture.number(),
        7.0,
        "a target-to-source-only binding preserves the authored ScriptInput value"
    );
}

#[test]
fn two_way_to_source_still_hydrates_the_script_input_from_source() {
    let fixture = number_fixture(
        9_508,
        ConverterFixture::None,
        DATA_BIND_TO_SOURCE | DATA_BIND_TWO_WAY,
    );
    assert!(fixture.model.set_number("source", 42.0));
    fixture.bind();
    assert_eq!(fixture.number(), 42.0);
}

#[test]
fn explicit_unresolved_converter_is_the_cpp_null_converter_passthrough() {
    let fixture = number_fixture(9_506, ConverterFixture::Unresolved, 0);
    assert!(fixture.model.set_number("source", 3.0));
    fixture.bind();
    assert!(
        fixture
            .bind_handle()
            .with(|bind| bind.as_data_bind().unwrap().converter())
            .flatten()
            .is_none()
    );
    assert_eq!(fixture.number(), 3.0);
}

#[test]
fn scripted_data_converter_without_a_script_uses_the_pinned_missing_state_branch() {
    let fixture = number_fixture(9_504, ConverterFixture::Scripted, 0);
    assert!(fixture.model.set_number("source", 3.0));
    fixture.bind();
    // The removed stateless facade rejected this. Pinned
    // scripted_data_converter.cpp::applyConversion explicitly returns input
    // when state()==nullptr or m_self==0; this is not a Rust fallback.
    let converter = fixture
        .bind_handle()
        .with(|bind| bind.as_data_bind().unwrap().converter())
        .flatten()
        .unwrap();
    assert_eq!(
        converter.core_type(),
        Some(type_key("ScriptedDataConverter"))
    );
    assert_eq!(fixture.number(), 3.0);
}

#[test]
fn stateful_interpolator_retains_occurrence_state() {
    let fixture = number_fixture(9_505, ConverterFixture::Interpolator, 0);
    assert!(fixture.model.set_number("source", 3.0));
    fixture.bind();
    // DataConverterInterpolator::convert resets its actual advancer to input
    // on its first run. The old facade's "requires retained state" rejection
    // is superseded by the now-native retained converter.
    assert_eq!(fixture.number(), 3.0);
    fixture.machine.advance_and_apply(0.1);
    fixture.machine.advance_and_apply(0.1);
    assert!(fixture.model.set_number("source", 5.0));
    fixture.machine.advance_and_apply(0.0);
    assert_eq!(
        fixture.number(),
        3.0,
        "changing the source is not a stateless passthrough"
    );
    fixture.machine.advance_and_apply(0.5);
    assert!((fixture.number() - 4.0).abs() < 1e-6);
    fixture.machine.advance_and_apply(0.5);
    assert_eq!(fixture.number(), 5.0);
}
