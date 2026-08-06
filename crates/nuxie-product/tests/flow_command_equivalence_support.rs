#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use nuxie::{
    File, RecordingFactory,
    command_queue::{
        CommandDataType, CommandEvent, CommandQueue, CommandValue, Listener,
        ViewModelInstanceHandle,
    },
    command_server::CommandServer,
};
use nuxie_product::flow_session::{
    FlowInstanceRef, FlowOperation, FlowOutputPhase, FlowQuery, FlowScalarValue, FlowSession,
    FlowSessionConfig, FlowSessionErrorKind, FlowStateBatch, FlowStateMutation, FlowValue,
    FlowValueArena,
};
use nuxie_schema::definition_by_name;

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
        .unwrap_or_else(|| panic!("fixture property {type_name}.{property_name} exists"))
        .key
        .int
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(
            definition_by_name(type_name)
                .expect("fixture definition exists")
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

fn push_string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

fn fixture_bytes() -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 16_310);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "ViewModel", |bytes| {
        push_string(bytes, "ViewModel", "name", "Root");
    });
    push_object(&mut bytes, "ViewModelPropertyBoolean", |bytes| {
        push_string(bytes, "ViewModelPropertyBoolean", "name", "Test Bool");
    });
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ViewModelInstance", |bytes| {
        push_string(bytes, "ViewModelInstance", "name", "Root default");
        push_uint(bytes, "ViewModelInstance", "viewModelId", 0);
    });
    push_object(&mut bytes, "ViewModelInstanceBoolean", |bytes| {
        push_uint(bytes, "ViewModelInstanceBoolean", "viewModelPropertyId", 0);
        push_uint(bytes, "ViewModelInstanceBoolean", "propertyValue", 0);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
        push_f32(bytes, "Artboard", "height", 100.0);
        push_uint(bytes, "Artboard", "viewModelId", 0);
    });
    bytes
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    Equivalent,
    NonEquivalent,
    Deferred,
}

#[derive(Debug, Clone, Copy)]
pub struct ResponsibilityDecision {
    pub responsibility: &'static str,
    pub classification: Classification,
    pub evidence: &'static str,
}

pub const RESPONSIBILITY_DECISIONS: &[ResponsibilityDecision] = &[
    ResponsibilityDecision {
        responsibility: "scalar value mutation",
        classification: Classification::Equivalent,
        evidence: "The shared fixture round-trips the same boolean value through both APIs.",
    },
    ResponsibilityDecision {
        responsibility: "output phases",
        classification: Classification::NonEquivalent,
        evidence: "Flow returns typed, sequenced phases synchronously; CommandServer emits events only after a server poll and client message dispatch.",
    },
    ResponsibilityDecision {
        responsibility: "atomic rollback",
        classification: Classification::NonEquivalent,
        evidence: "Flow rejects a mixed-validity batch without its first write; CommandServer preserves an earlier successful command after a later command fails.",
    },
    ResponsibilityDecision {
        responsibility: "wake scheduling",
        classification: Classification::NonEquivalent,
        evidence: "FlowResult owns an explicit wake_after_seconds deadline; CommandQueue has blocking command availability and settled events, not an equivalent product wake contract.",
    },
    ResponsibilityDecision {
        responsibility: "terminal errors",
        classification: Classification::NonEquivalent,
        evidence: "Flow terminally poisons a session after post-mutation result failure; CommandServer reports a command error and processes later commands.",
    },
    ResponsibilityDecision {
        responsibility: "wasm suitability",
        classification: Classification::NonEquivalent,
        evidence: "Flow is a caller-driven synchronous object; CommandQueue's protocol owns Mutex, Condvar, and a separately driven server loop.",
    },
    ResponsibilityDecision {
        responsibility: "latency",
        classification: Classification::Deferred,
        evidence: "The companion measurement binary records both paths, but performance alone cannot establish semantic substitutability.",
    },
    ResponsibilityDecision {
        responsibility: "allocations",
        classification: Classification::Deferred,
        evidence: "The companion measurement binary records process allocations; the count is diagnostic and platform-dependent.",
    },
    ResponsibilityDecision {
        responsibility: "Flow-only graph and host-cycle machinery",
        classification: Classification::NonEquivalent,
        evidence: "Graph cloning, transaction transfer, settlement, and host-effect cycles implement the rollback and commit boundary absent from CommandServer.",
    },
];

pub struct ScalarRoundTripComparison {
    pub flow_value: bool,
    pub command_value: bool,
}

pub struct DeliveryPhaseComparison {
    pub flow_outputs_before_return: usize,
    pub command_events_before_server_poll: usize,
    pub command_events_before_message_dispatch: usize,
    pub command_events_after_message_dispatch: usize,
}

pub struct AtomicFailureComparison {
    pub flow_value_after_failure: bool,
    pub command_value_after_failure: bool,
    pub flow_error_class: &'static str,
    pub command_error_count: usize,
}

struct CommandFixture {
    queue: CommandQueue,
    server: CommandServer,
    root: ViewModelInstanceHandle,
    events: Arc<Mutex<Vec<CommandEvent>>>,
    _listener: Listener,
}

impl CommandFixture {
    fn new() -> Self {
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        let listener: Listener = Arc::new(move |event: &CommandEvent| {
            sink.lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(event.clone());
        });
        let queue = CommandQueue::new();
        let file = queue.load_file(fixture_bytes(), None, 0);
        let artboard = queue.instantiate_default_artboard(file, None, 0);
        let root = queue.instantiate_view_model_for_artboard(
            file,
            artboard,
            Some(String::new()),
            Some(&listener),
            0,
        );
        let mut server = CommandServer::new(queue.clone(), Box::new(RecordingFactory::new()));
        assert!(server.process_commands());
        queue.process_messages();
        events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
        Self {
            queue,
            server,
            root,
            events,
            _listener: listener,
        }
    }

    fn event_snapshot(&self) -> Vec<CommandEvent> {
        self.events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn take_events(&self) -> Vec<CommandEvent> {
        let mut events = self
            .events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::mem::take(&mut *events)
    }
}

fn flow_fixture() -> (FlowSession, nuxie_product::flow_session::FlowInstanceId) {
    let file = Arc::new(
        File::import(&fixture_bytes()).expect("import shared synthetic equivalence fixture"),
    );
    let (session, bootstrap) =
        FlowSession::create(file, FlowSessionConfig::default()).expect("create Flow fixture");
    let root = bootstrap
        .catalog
        .root_instance_id
        .expect("shared fixture has a root view model");
    (session, root)
}

fn flow_bool(arena: &FlowValueArena) -> bool {
    let (_, root_id) = arena.roots.first().expect("root value");
    let root = arena
        .nodes
        .iter()
        .find(|node| node.id == *root_id)
        .expect("root node");
    let FlowValue::ViewModel(properties) = &root.value else {
        panic!("root must be a view model")
    };
    let (_, bool_id) = properties
        .iter()
        .find(|(name, _)| name == "Test Bool")
        .expect("Test Bool property");
    let value = arena
        .nodes
        .iter()
        .find(|node| node.id == *bool_id)
        .expect("Test Bool node");
    let FlowValue::Bool(value) = value.value else {
        panic!("Test Bool must be boolean")
    };
    value
}

fn query_flow_bool(session: &mut FlowSession) -> bool {
    let values = session
        .perform(FlowOperation::Query(FlowQuery::Values))
        .expect("query Flow values")
        .values
        .expect("Flow values result");
    flow_bool(&values)
}

fn command_bool(events: &[CommandEvent], request_id: u64) -> bool {
    events
        .iter()
        .find_map(|event| match event {
            CommandEvent::ViewModelValue {
                request_id: candidate,
                path,
                value: CommandValue::Boolean(value),
                ..
            } if *candidate == request_id && path == "Test Bool" => Some(*value),
            _ => None,
        })
        .expect("boolean command result")
}

pub fn compare_scalar_round_trip() -> ScalarRoundTripComparison {
    let (mut flow, root) = flow_fixture();
    flow.perform(FlowOperation::StateBatch(FlowStateBatch {
        host_mutation_id: Some(1),
        mutations: vec![FlowStateMutation::SetValue {
            instance: FlowInstanceRef::Existing(root),
            path: "Test Bool".to_owned(),
            value: FlowScalarValue::Bool(true),
        }],
        new_instances: Vec::new(),
    }))
    .expect("Flow scalar mutation");

    let mut command = CommandFixture::new();
    command
        .queue
        .set_view_model_value(command.root, "Test Bool", CommandValue::Boolean(true), 1);
    command
        .queue
        .request_view_model_value(command.root, "Test Bool", CommandDataType::Boolean, 2);
    assert!(command.server.process_commands());
    command.queue.process_messages();

    ScalarRoundTripComparison {
        flow_value: query_flow_bool(&mut flow),
        command_value: command_bool(&command.event_snapshot(), 2),
    }
}

pub fn compare_delivery_phases() -> DeliveryPhaseComparison {
    let (mut flow, root) = flow_fixture();
    let result = flow
        .perform(FlowOperation::StateBatch(FlowStateBatch {
            host_mutation_id: Some(3),
            mutations: vec![FlowStateMutation::SetValue {
                instance: FlowInstanceRef::Existing(root),
                path: "Test Bool".to_owned(),
                value: FlowScalarValue::Bool(true),
            }],
            new_instances: Vec::new(),
        }))
        .expect("Flow scalar mutation");
    assert!(
        result
            .outputs
            .iter()
            .all(|output| output.phase == FlowOutputPhase::ViewModelChanges)
    );

    let mut command = CommandFixture::new();
    command
        .queue
        .set_view_model_value(command.root, "Test Bool", CommandValue::Boolean(true), 3);
    command
        .queue
        .request_view_model_value(command.root, "Test Bool", CommandDataType::Boolean, 4);
    let before_server = command.event_snapshot().len();
    assert!(command.server.process_commands());
    let before_dispatch = command.event_snapshot().len();
    command.queue.process_messages();
    let after_dispatch = command.event_snapshot().len();

    DeliveryPhaseComparison {
        flow_outputs_before_return: result.outputs.len(),
        command_events_before_server_poll: before_server,
        command_events_before_message_dispatch: before_dispatch,
        command_events_after_message_dispatch: after_dispatch,
    }
}

pub fn compare_atomic_failure() -> AtomicFailureComparison {
    let (mut flow, root) = flow_fixture();
    let flow_error = flow
        .perform(FlowOperation::StateBatch(FlowStateBatch {
            host_mutation_id: Some(5),
            mutations: vec![
                FlowStateMutation::SetValue {
                    instance: FlowInstanceRef::Existing(root),
                    path: "Test Bool".to_owned(),
                    value: FlowScalarValue::Bool(true),
                },
                FlowStateMutation::SetValue {
                    instance: FlowInstanceRef::Existing(root),
                    path: "Missing Bool".to_owned(),
                    value: FlowScalarValue::Bool(false),
                },
            ],
            new_instances: Vec::new(),
        }))
        .expect_err("mixed-validity Flow batch must fail");

    let mut command = CommandFixture::new();
    command
        .queue
        .set_view_model_value(command.root, "Test Bool", CommandValue::Boolean(true), 5);
    command
        .queue
        .set_view_model_value(command.root, "Test Bool", CommandValue::Number(1.0), 6);
    command
        .queue
        .request_view_model_value(command.root, "Test Bool", CommandDataType::Boolean, 7);
    assert!(command.server.process_commands());
    command.queue.process_messages();
    let events = command.event_snapshot();

    AtomicFailureComparison {
        flow_value_after_failure: query_flow_bool(&mut flow),
        command_value_after_failure: command_bool(&events, 7),
        flow_error_class: match flow_error.kind() {
            FlowSessionErrorKind::NotFound => "not_found",
            _ => "unexpected",
        },
        command_error_count: events
            .iter()
            .filter(|event| matches!(event, CommandEvent::ViewModelError { .. }))
            .count(),
    }
}

pub fn run_flow_scalar_iterations(iterations: usize) -> bool {
    let (mut flow, root) = flow_fixture();
    let mut value = false;
    for index in 0..iterations {
        value = index % 2 == 0;
        flow.perform(FlowOperation::StateBatch(FlowStateBatch {
            host_mutation_id: Some(index as u64),
            mutations: vec![FlowStateMutation::SetValue {
                instance: FlowInstanceRef::Existing(root),
                path: "Test Bool".to_owned(),
                value: FlowScalarValue::Bool(value),
            }],
            new_instances: Vec::new(),
        }))
        .expect("measured Flow mutation");
        assert_eq!(query_flow_bool(&mut flow), value);
    }
    value
}

pub fn run_command_scalar_iterations(iterations: usize) -> bool {
    let mut command = CommandFixture::new();
    let mut value = false;
    for index in 0..iterations {
        value = index % 2 == 0;
        let request_id = index as u64 + 1;
        command.queue.set_view_model_value(
            command.root,
            "Test Bool",
            CommandValue::Boolean(value),
            request_id,
        );
        command.queue.request_view_model_value(
            command.root,
            "Test Bool",
            CommandDataType::Boolean,
            request_id,
        );
        assert!(command.server.process_commands());
        command.queue.process_messages();
        assert_eq!(command_bool(&command.take_events(), request_id), value);
    }
    value
}
