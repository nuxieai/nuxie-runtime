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

fn scripted_listener_file(protocol_source: &[u8], action_count: usize) -> Vec<u8> {
    scripted_listener_file_with_inputs(protocol_source, action_count, |_, _| {})
}

fn scripted_listener_file_with_inputs(
    protocol_source: &[u8],
    action_count: usize,
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
    let instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))
        .expect("instantiate scripted listener artboard");
    let machine = instance
        .default_state_machine_instance()
        .expect("instantiate scripted listener state machine");
    (file, instance, machine, RecordingFactory::new())
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
            local function verify(self, amount)
                local root = context:viewModel()
                if root == nil or root.amount.value ~= amount then
                    error("Context.viewModel is stale")
                end
                if self.boundAmount ~= amount then
                    error("bound scalar is stale")
                end
                if self.boundChild == nil or self.boundChild.score.value ~= 4 then
                    error("bound nested view model is missing")
                end
            end
            return {
                init = function(self, initContext)
                    if context ~= initContext then error("init Context changed") end
                    verify(self, 9.5)
                    nuxie.trigger("bound_init_ready")
                    return true
                end,
                boundPulse = function(self)
                    verify(self, 17)
                    nuxie.trigger("bound_trigger_fired")
                end,
                performAction = function(self, _invocation)
                    verify(self, 17)
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

struct InertScriptInstance;

impl ScriptInstance for InertScriptInstance {
    fn has_method(&self, _method: ScriptMethod) -> std::result::Result<bool, ScriptError> {
        Ok(false)
    }

    fn call_method(
        &mut self,
        _method: ScriptMethod,
        _args: &[ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<ScriptValue, ScriptError> {
        Ok(ScriptValue::Nil)
    }

    fn get_input(&self, _name: &str) -> std::result::Result<ScriptValue, ScriptError> {
        Ok(ScriptValue::Nil)
    }

    fn set_input(
        &mut self,
        _name: &str,
        _value: ScriptValue,
    ) -> std::result::Result<(), ScriptError> {
        Ok(())
    }
}

#[test]
fn listener_init_failure_aborts_the_whole_attachment_batch() {
    let (file, mut instance, mut machine, mut factory) = prepared_machine(
        br#"
            local generated = 0

            return function(_context)
                generated += 1
                local occurrence = generated
                return {
                    init = function(_self, _context)
                        return occurrence == 1
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

    let error = instantiate_script_listener_actions(&file, &mut machine, &mut factory, None)
        .expect_err("the second listener init must reject the attachment batch");
    assert!(
        error.message().contains("init returned false or nil"),
        "unexpected listener init error: {error}"
    );

    assert_eq!(action_ids.len(), 2);
    for action_id in action_ids {
        machine
            .set_scripted_listener_action_instance(action_id, Box::new(InertScriptInstance))
            .expect("a failed batch must leave every listener action unattached");
    }
}

#[test]
fn listener_failed_input_hydration_leaves_the_whole_batch_unattached() {
    let bytes = scripted_listener_file_with_inputs(
        br#"
            return function(_context)
                return {
                    init = function(_self, _initContext) return true end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
        2,
        |action_index, bytes| {
            if action_index != 1 {
                return;
            }
            let mut path = Vec::new();
            push_var_uint(&mut path, 0);
            push_object(bytes, "ScriptInputViewModelProperty", |bytes| {
                push_string(
                    bytes,
                    "ScriptInputViewModelProperty",
                    "name",
                    "missingContext",
                );
                push_blob(
                    bytes,
                    "ScriptInputViewModelProperty",
                    "dataBindPathIds",
                    &path,
                );
            });
        },
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
    let action_ids = machine
        .scripted_listener_actions()
        .iter()
        .map(|definition| definition.action_global_id())
        .collect::<Vec<_>>();

    let error = instantiate_script_listener_actions(&file, &mut machine, &mut factory, None)
        .expect_err("view-model input without a root context must reject hydration");
    assert!(
        error
            .message()
            .contains("requires an active root view-model context"),
        "unexpected hydration error: {error}"
    );
    for action_id in action_ids {
        machine
            .set_scripted_listener_action_instance(action_id, Box::new(InertScriptInstance))
            .expect("failed hydration must leave every occurrence unattached");
    }
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
